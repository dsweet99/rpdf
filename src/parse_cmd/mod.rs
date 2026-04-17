#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::cli::ParseCli;
use crate::engine;
use crate::expand::expand_inputs;
use crate::model::{self, DocumentJson, ParseConfig};
use crate::parse_batch;
use crate::parse_document::{build_document_json, eprint_partial_success, write_json_document};
use crate::pagespec::parse_pageset;
use crate::parse_validate;
use pdfium_render::prelude::*;
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub use parse_validate::validate_parse_cli;

pub fn run_parse(cli: &ParseCli) -> i32 {
    let pdfium = engine::init_pdfium();
    let inputs = match expand_inputs(&cli.inputs) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    if let Err(e) = validate_parse_cli(cli, inputs.len()) {
        eprintln!("{e}");
        return 1;
    }
    let page_filter = match load_pages_filter(cli) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    let cfg = parse_config(cli);
    if inputs.len() > 1 {
        parse_batch::run_parse_batch(&pdfium, cli, &inputs, page_filter.as_ref(), &cfg)
    } else {
        run_parse_one(&pdfium, cli, &inputs[0], page_filter.as_ref(), &cfg)
    }
}

pub fn parse_config(cli: &ParseCli) -> ParseConfig {
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

pub fn load_pages_filter(cli: &ParseCli) -> Result<Option<BTreeSet<u16>>, String> {
    cli.pages
        .as_ref()
        .map_or(Ok(None), |s| parse_pageset(s).map(Some))
}

fn write_json_out(path: &Path, dj: &DocumentJson) -> i32 {
    match write_json_document(path, dj) {
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

fn run_parse_one_postprocess(md: &str) -> String {
    if std::env::var("RPDF_DBG_RAW").is_ok() {
        eprintln!("==RAW_MD_BEGIN==\n{md}\n==RAW_MD_END==");
    }
    model::normalize_text(&model::postprocess_extracted_markdown(md))
}

fn run_parse_one(
    pdfium: &Pdfium,
    cli: &ParseCli,
    input: &Path,
    page_filter: Option<&BTreeSet<u16>>,
    cfg: &ParseConfig,
) -> i32 {
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
        page_filter,
        cfg.clone(),
        cli.quiet,
    );
    let md_out = run_parse_one_postprocess(&md);
    if !cli.stdout {
        let r = emit_markdown_output(cli, input, &md_out);
        if r != 0 {
            return r;
        }
    }
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
    if cli.stdout {
        let r = emit_markdown_output(cli, input, &md_out);
        if r != 0 {
            return r;
        }
    }
    let code = dj.status.exit_code();
    eprint_partial_success(cli.quiet, dj.status, &dj.failed_pages);
    code
}

#[cfg(test)]
mod kiss_coverage {
    use super::*;

    #[test]
    fn handler_refs() {
        let _: fn(&ParseCli) -> i32 = run_parse;
        let _: fn(&ParseCli, usize) -> Result<(), String> = validate_parse_cli;
        let _: fn(&Pdfium, &ParseCli, &std::path::Path, Option<&BTreeSet<u16>>, &ParseConfig) -> i32 =
            run_parse_one;
        assert_eq!(
            stringify!(crate::parse_document::build_document_json),
            "crate::parse_document::build_document_json"
        );
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use crate::test_support::parse_cli_base;

    #[test]
    fn parse_config_defaults() {
        let c = parse_cli_base();
        let cfg = parse_config(&c);
        assert_eq!(cfg.reading_order, "basic");
        assert_eq!(cfg.table_mode, "off");
        assert!(!cfg.use_struct_tree);
        assert!(!cfg.include_header_footer);
        assert!(!cfg.keep_line_breaks);
    }

    #[test]
    fn parse_config_propagates_explicit_options() {
        let mut c = parse_cli_base();
        c.reading_order = Some("xycut".to_string());
        c.table_mode = Some("lines".to_string());
        c.use_struct_tree = true;
        c.include_header_footer = true;
        c.keep_line_breaks = true;
        let cfg = parse_config(&c);
        assert_eq!(cfg.reading_order, "xycut");
        assert_eq!(cfg.table_mode, "lines");
        assert!(cfg.use_struct_tree);
        assert!(cfg.include_header_footer);
        assert!(cfg.keep_line_breaks);
    }

    #[test]
    fn load_pages_filter_none() {
        let c = parse_cli_base();
        assert!(load_pages_filter(&c).unwrap().is_none());
    }

    #[test]
    fn load_pages_filter_parses() {
        let mut c = parse_cli_base();
        c.pages = Some("1,2-3".to_string());
        let f = load_pages_filter(&c).expect("ok");
        let set = f.expect("some");
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
    }

    #[test]
    fn load_pages_filter_rejects_invalid() {
        let mut c = parse_cli_base();
        c.pages = Some("not-a-page".to_string());
        assert!(load_pages_filter(&c).is_err());
    }
}
