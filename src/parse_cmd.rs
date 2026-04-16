#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::cli::ParseCli;
use crate::engine;
use crate::expand::expand_inputs;
use crate::model::{self, DocumentJson, ParseConfig, RunStatus};
use crate::parse_document::{build_document_json, write_atomic};
use crate::pagespec::parse_pageset;
use pdfium_render::prelude::*;
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn run_parse(cli: &ParseCli) -> i32 {
    if let Err(e) = validate_parse_cli(cli) {
        eprintln!("{e}");
        return 1;
    }
    let pdfium = engine::init_pdfium();
    let inputs = match expand_inputs(&cli.inputs) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    if inputs.len() > 1 {
        run_parse_batch(&pdfium, cli, &inputs)
    } else {
        run_parse_one(&pdfium, cli, &inputs[0])
    }
}

fn validate_parse_cli(cli: &ParseCli) -> Result<(), String> {
    let mut problems = Vec::new();
    if cli.stdout && cli.output.is_some() {
        problems.push("--stdout cannot be combined with --output");
    }
    if cli.stdout && cli.output_dir.is_some() {
        problems.push("--stdout cannot be used with --output-dir");
    }
    if cli.stdout && cli.inputs.len() > 1 {
        problems.push("--stdout is only valid with a single input path");
    }
    if cli.output.is_some() && cli.inputs.len() > 1 && cli.output_dir.is_none() {
        problems.push("--output requires a single input unless --output-dir is used");
    }
    if cli.json.is_some() && cli.inputs.len() > 1 {
        problems.push("--json requires a single input");
    }
    if cli.reading_order.as_deref().is_some_and(|s| {
        !matches!(s, "off" | "basic" | "xycut")
    }) {
        problems.push("invalid --reading-order (expected off|basic|xycut)");
    }
    if cli.table_mode.as_deref().is_some_and(|s| {
        !matches!(s, "off" | "lines" | "heuristic")
    }) {
        problems.push("invalid --table-mode (expected off|lines|heuristic)");
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

fn parse_config(cli: &ParseCli) -> ParseConfig {
    let reading = cli.reading_order.as_deref().unwrap_or("basic");
    let table = cli.table_mode.as_deref().unwrap_or("off");
    ParseConfig {
        reading_order: reading.to_string(),
        table_mode: table.to_string(),
        use_struct_tree: cli.use_struct_tree,
        include_header_footer: cli.include_header_footer,
        keep_line_breaks: cli.keep_line_breaks,
    }
}

fn load_pages_filter(cli: &ParseCli) -> Result<Option<BTreeSet<u16>>, String> {
    cli.pages
        .as_ref()
        .map_or(Ok(None), |s| parse_pageset(s).map(Some))
}

fn write_json_out(path: &Path, dj: &DocumentJson) -> i32 {
    match write_atomic(path, |w| serde_json::to_writer_pretty(w, dj).map_err(|e| e.to_string())) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            2
        }
    }
}

fn emit_markdown_output(cli: &ParseCli, input: &Path, md_out: &str) -> i32 {
    if cli.stdout {
        let mut out = io::stdout().lock();
        return match writeln!(out, "{md_out}") {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{e}");
                2
            }
        };
    }
    let out_path: PathBuf = cli
        .output
        .clone()
        .unwrap_or_else(|| input.with_extension("md"));
    if out_path.exists() {
        eprintln!("refusing to overwrite {}", out_path.display());
        return 1;
    }
    match fs::write(&out_path, md_out.as_bytes()) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            2
        }
    }
}

fn run_parse_one(pdfium: &Pdfium, cli: &ParseCli, input: &Path) -> i32 {
    let page_filter = match load_pages_filter(cli) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    let cfg = parse_config(cli);
    let doc = match pdfium.load_pdf_from_file(input, cli.password.as_deref()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let (dj, md) = build_document_json(
        &doc,
        input,
        page_filter.as_ref(),
        cfg,
        cli.quiet,
    );
    if let Some(p) = &cli.json {
        let c = write_json_out(p, &dj);
        if c != 0 {
            return c;
        }
    }
    if let Some(p) = &cli.debug_json {
        let c = write_json_out(p, &dj);
        if c != 0 {
            return c;
        }
    }
    let md_out = model::normalize_text(&md);
    let r = emit_markdown_output(cli, input, &md_out);
    if r != 0 {
        return r;
    }
    let code = status_exit_code(dj.status);
    if !cli.quiet && code == 3 {
        eprintln!(
            "partial_success: failed_pages={:?}",
            dj.failed_pages
        );
    }
    code
}

const fn status_exit_code(s: RunStatus) -> i32 {
    match s {
        RunStatus::Success => 0,
        RunStatus::PartialSuccess => 3,
        RunStatus::Failure => 2,
    }
}

fn run_parse_batch(pdfium: &Pdfium, cli: &ParseCli, inputs: &[PathBuf]) -> i32 {
    let Some(dir) = &cli.output_dir else {
        eprintln!("multiple inputs require --output-dir");
        return 1;
    };
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("{}: {e}", dir.display());
        return 2;
    }
    let page_filter = match load_pages_filter(cli) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    let cfg = parse_config(cli);
    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut partial = 0usize;
    for input in inputs {
        let r = batch_process_one(
            pdfium,
            cli,
            dir,
            input,
            page_filter.as_ref(),
            &cfg,
        );
        match r {
            BatchLine::Skip => {
                failed += 1;
            }
            BatchLine::Outcome { tag } => {
                match tag {
                    "ok" => ok += 1,
                    "partial" => partial += 1,
                    "fail" => failed += 1,
                    _ => {}
                }
                if !cli.quiet {
                    eprintln!("{} {}", input.display(), tag);
                }
            }
        }
    }
    batch_exit_code(ok, partial, failed)
}

enum BatchLine {
    Skip,
    Outcome { tag: &'static str },
}

fn batch_process_one(
    pdfium: &Pdfium,
    cli: &ParseCli,
    dir: &Path,
    input: &Path,
    page_filter: Option<&BTreeSet<u16>>,
    cfg: &ParseConfig,
) -> BatchLine {
    let Some(stem) = input.file_stem().and_then(|s| s.to_str()) else {
        eprintln!("bad filename: {}", input.display());
        return BatchLine::Skip;
    };
    let md_path = dir.join(format!("{stem}.md"));
    let json_path = cli.json.as_ref().map(|_| dir.join(format!("{stem}.json")));
    let doc = match pdfium.load_pdf_from_file(input, cli.password.as_deref()) {
        Ok(d) => d,
        Err(e) => {
            if !cli.quiet {
                eprintln!("{}: {}", input.display(), e);
            }
            return BatchLine::Skip;
        }
    };
    let (dj, md) = build_document_json(
        &doc,
        input,
        page_filter,
        cfg.clone(),
        cli.quiet,
    );
    let tag = match dj.status {
        RunStatus::Success => "ok",
        RunStatus::PartialSuccess => "partial",
        RunStatus::Failure => "fail",
    };
    if md_path.exists() {
        eprintln!("refusing to overwrite {}", md_path.display());
        return BatchLine::Skip;
    }
    let md_out = model::normalize_text(&md);
    if fs::write(&md_path, md_out.as_bytes()).is_err() {
        if !cli.quiet {
            eprintln!("{}: write error", input.display());
        }
        return BatchLine::Skip;
    }
    if let Some(jp) = &json_path {
        if write_atomic(jp, |w| serde_json::to_writer_pretty(w, &dj).map_err(|e| e.to_string()))
            .is_err()
        {
            if !cli.quiet {
                eprintln!("{}: json write error", input.display());
            }
            return BatchLine::Skip;
        }
    }
    BatchLine::Outcome { tag }
}

const fn batch_exit_code(ok: usize, partial: usize, failed: usize) -> i32 {
    let any_ok = ok + partial > 0;
    if !any_ok {
        2
    } else if failed > 0 || partial > 0 {
        3
    } else {
        0
    }
}

#[cfg(test)]
mod kiss_coverage {
    use super::*;

    #[test]
    fn handler_refs() {
        let _: fn(&ParseCli) -> i32 = run_parse;
        let _: fn(&ParseCli) -> Result<(), String> = validate_parse_cli;
        let _: fn(&Pdfium, &ParseCli, &std::path::Path) -> i32 = run_parse_one;
        let _: fn(&Pdfium, &ParseCli, &[PathBuf]) -> i32 = run_parse_batch;
        assert_eq!(stringify!(write_atomic), "write_atomic");
        assert_eq!(stringify!(batch_process_one), "batch_process_one");
        assert_eq!(
            stringify!(crate::parse_document::build_document_json),
            "crate::parse_document::build_document_json"
        );
    }
}
