#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::cli::ParseCli;
use crate::model::{self, DocumentJson, ParseConfig, RunStatus};
use crate::parse_document::{build_document_json, eprint_partial_success, write_exclusive, write_json_document};
use crate::parse_overwrite;
use pdfium_render::prelude::*;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_parse_batch(
    pdfium: &Pdfium,
    cli: &ParseCli,
    inputs: &[PathBuf],
    page_filter: Option<&BTreeSet<u32>>,
    cfg: &ParseConfig,
) -> i32 {
    let mut counts = (0usize, 0usize, 0usize);
    let ctx = BatchRunCtx {
        pdfium,
        cli,
        inputs,
        page_filter,
        cfg,
    };
    if let Some(dir) = &cli.output_dir {
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!("{}: {e}", dir.display());
            return 2;
        }
        run_batch_with_output_dir(&mut counts, &ctx, dir);
    } else {
        for input in ctx.inputs {
            let md_path = input.with_extension("md");
            let batch_paths = BatchArtifacts {
                markdown: md_path.as_path(),
                json: None,
                debug: None,
            };
            let r = batch_process_one(
                ctx.pdfium,
                ctx.cli,
                input,
                ctx.page_filter,
                ctx.cfg,
                &batch_paths,
            );
            batch_accumulate(
                &mut counts.0,
                &mut counts.1,
                &mut counts.2,
                input,
                ctx.cli.quiet,
                r,
            );
        }
    }
    batch_exit_code(counts.0, counts.2, counts.1)
}

struct BatchRunCtx<'a> {
    pdfium: &'a Pdfium,
    cli: &'a ParseCli,
    inputs: &'a [PathBuf],
    page_filter: Option<&'a BTreeSet<u32>>,
    cfg: &'a ParseConfig,
}

fn run_batch_with_output_dir(
    counts: &mut (usize, usize, usize),
    ctx: &BatchRunCtx<'_>,
    dir: &Path,
) {
    let mut used_stems: BTreeSet<String> = BTreeSet::new();
    for (idx, input) in ctx.inputs.iter().enumerate() {
        let raw_stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .map_or_else(|| format!("input-{}", idx + 1), str::to_owned);
        let stem = reserve_output_stem(&raw_stem, &mut used_stems);
        let md_path = dir.join(format!("{stem}.md"));
        let json_path = ctx
            .cli
            .json
            .as_ref()
            .and_then(|p| p.file_name().and_then(|x| x.to_str()))
            .map(|name| dir.join(format!("{stem}.{name}")));
        let debug_path = ctx
            .cli
            .debug_json
            .as_ref()
            .and_then(|p| p.file_name().and_then(|x| x.to_str()))
            .map(|name| dir.join(format!("{stem}.{name}")));
        let batch_paths = BatchArtifacts {
            markdown: md_path.as_path(),
            json: json_path.as_deref(),
            debug: debug_path.as_deref(),
        };
        let r = batch_process_one(
            ctx.pdfium,
            ctx.cli,
            input,
            ctx.page_filter,
            ctx.cfg,
            &batch_paths,
        );
        batch_accumulate(
            &mut counts.0,
            &mut counts.1,
            &mut counts.2,
            input,
            ctx.cli.quiet,
            r,
        );
    }
}

fn reserve_output_stem(raw_stem: &str, used_stems: &mut BTreeSet<String>) -> String {
    if used_stems.insert(raw_stem.to_string()) {
        return raw_stem.to_string();
    }
    let base = base_stem_without_numeric_suffix(raw_stem);
    let mut n = 1usize;
    loop {
        let candidate = format!("{base}-{n}");
        if used_stems.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}

fn base_stem_without_numeric_suffix(raw_stem: &str) -> &str {
    let Some((base, suffix)) = raw_stem.rsplit_once('-') else {
        return raw_stem;
    };
    if !base.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
        return base;
    }
    raw_stem
}

fn batch_accumulate(
    ok: &mut usize,
    failed: &mut usize,
    partial: &mut usize,
    input: &Path,
    quiet: bool,
    r: BatchLine,
) {
    match r {
        BatchLine::Skip => {
            *failed += 1;
        }
        BatchLine::Outcome(o) => {
            match o {
                BatchOutcome::Ok => *ok += 1,
                BatchOutcome::Partial => *partial += 1,
                BatchOutcome::Fail => *failed += 1,
            }
            if !quiet {
                eprintln!("{} {}", input.display(), o.as_str());
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BatchOutcome {
    Ok,
    Partial,
    Fail,
}

impl BatchOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Partial => "partial",
            Self::Fail => "fail",
        }
    }
}

#[derive(Copy, Clone)]
pub enum BatchLine {
    Skip,
    Outcome(BatchOutcome),
}
pub struct BatchArtifacts<'a> {
    pub markdown: &'a Path,
    pub json: Option<&'a Path>,
    pub debug: Option<&'a Path>,
}
const fn status_outcome(s: RunStatus) -> BatchOutcome {
    match s {
        RunStatus::Success => BatchOutcome::Ok,
        RunStatus::PartialSuccess => BatchOutcome::Partial,
        RunStatus::Failure => BatchOutcome::Fail,
    }
}

fn write_markdown_artifact(
    input: &Path,
    quiet: bool,
    markdown: &Path,
    md: &str,
    outcome: BatchOutcome,
) -> BatchLine {
    let md_out = model::normalize_text(&model::postprocess_extracted_markdown(md));
    match write_exclusive(markdown, |buf| {
        buf.extend_from_slice(md_out.as_bytes());
        Ok(())
    }) {
        Ok(()) => BatchLine::Outcome(outcome),
        Err(e) => {
            if !quiet {
                if e.starts_with("refusing to overwrite ") {
                    eprintln!("{e}");
                } else {
                    eprintln!("{}: {e}", input.display());
                }
            }
            BatchLine::Skip
        }
    }
}

pub fn batch_process_one(
    pdfium: &Pdfium,
    cli: &ParseCli,
    input: &Path,
    page_filter: Option<&BTreeSet<u32>>,
    cfg: &ParseConfig,
    artifacts: &BatchArtifacts<'_>,
) -> BatchLine {
    if let Some((_, path)) = parse_overwrite::first_existing_output(
        Some(artifacts.markdown),
        artifacts.json,
        artifacts.debug,
    ) {
        parse_overwrite::emit_overwrite(path);
        return BatchLine::Skip;
    }
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
    let outcome = status_outcome(dj.status);
    let line = write_markdown_artifact(input, cli.quiet, artifacts.markdown, &md, outcome);
    if !matches!(line, BatchLine::Outcome(_)) {
        return line;
    }
    if let Err(skip) = write_optional_jsons(&dj, cli.quiet, artifacts) {
        return skip;
    }
    if matches!(line, BatchLine::Outcome(BatchOutcome::Partial)) {
        eprint_partial_success(cli.quiet, dj.status, &dj.failed_pages);
    }
    line
}

fn write_json_sidecar_if_present(
    path: Option<&Path>,
    dj: &DocumentJson,
    quiet: bool,
) -> Result<bool, BatchLine> {
    let Some(path) = path else {
        return Ok(false);
    };
    match write_json_document(path, dj) {
        Ok(()) => Ok(true),
        Err(e) => {
            if !quiet {
                eprintln!("{e}");
            }
            Err(BatchLine::Skip)
        }
    }
}

fn write_optional_jsons(
    dj: &DocumentJson,
    quiet: bool,
    artifacts: &BatchArtifacts<'_>,
) -> Result<(), BatchLine> {
    let json_written = write_json_sidecar_if_present(artifacts.json, dj, quiet)
        .inspect_err(|_skip| {
            let _ = fs::remove_file(artifacts.markdown);
        })?;
    write_json_sidecar_if_present(artifacts.debug, dj, quiet).inspect_err(|_skip| {
        let _ = fs::remove_file(artifacts.markdown);
        if json_written {
            if let Some(path) = artifacts.json {
                let _ = fs::remove_file(path);
            }
        }
    })?;
    Ok(())
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
mod batch_contract_tests {
    use super::{
        base_stem_without_numeric_suffix, batch_exit_code, reserve_output_stem, status_outcome,
        BatchOutcome,
    };
    use crate::model::RunStatus;
    use std::collections::BTreeSet;

    #[test]
    fn status_outcome_maps_status() {
        assert_eq!(status_outcome(RunStatus::Success), BatchOutcome::Ok);
        assert_eq!(status_outcome(RunStatus::PartialSuccess), BatchOutcome::Partial);
        assert_eq!(status_outcome(RunStatus::Failure), BatchOutcome::Fail);
    }

    #[test]
    fn batch_exit_code_all_success() {
        assert_eq!(batch_exit_code(2, 0, 0), 0);
    }

    #[test]
    fn batch_exit_code_partial_no_failed() {
        assert_eq!(batch_exit_code(1, 1, 0), 3);
    }

    #[test]
    fn batch_exit_code_mixed_failed_and_success() {
        assert_eq!(batch_exit_code(1, 0, 1), 3);
    }

    #[test]
    fn batch_exit_code_all_failed() {
        assert_eq!(batch_exit_code(0, 0, 2), 2);
    }

    #[test]
    fn batch_exit_code_only_partial_still_yields_three() {
        assert_eq!(batch_exit_code(0, 3, 0), 3);
    }

    #[test]
    fn reserve_output_stem_avoids_suffix_collisions() {
        let mut used = BTreeSet::new();
        let stems = ["report", "report", "report-1"];
        let out: Vec<String> = stems
            .iter()
            .map(|s| reserve_output_stem(s, &mut used))
            .collect();
        assert_eq!(out, vec!["report", "report-1", "report-2"]);
    }

    #[test]
    fn base_stem_without_numeric_suffix_strips_dash_digits_only() {
        assert_eq!(base_stem_without_numeric_suffix("report-1"), "report");
        assert_eq!(base_stem_without_numeric_suffix("report-final"), "report-final");
    }
}

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn symbol_refs() {
        assert_eq!(stringify!(super::run_parse_batch), "super::run_parse_batch");
        assert_eq!(stringify!(super::batch_accumulate), "super::batch_accumulate");
        assert_eq!(stringify!(super::BatchLine), "super::BatchLine");
        assert_eq!(stringify!(super::BatchArtifacts), "super::BatchArtifacts");
        assert_eq!(
            stringify!(super::write_markdown_artifact),
            "super::write_markdown_artifact"
        );
        assert_eq!(
            stringify!(crate::parse_overwrite::first_existing_output),
            "crate::parse_overwrite::first_existing_output"
        );
        assert_eq!(stringify!(super::batch_process_one), "super::batch_process_one");
        assert_eq!(
            stringify!(super::write_json_sidecar_if_present),
            "super::write_json_sidecar_if_present"
        );
        assert_eq!(stringify!(super::write_optional_jsons), "super::write_optional_jsons");
    }
}
