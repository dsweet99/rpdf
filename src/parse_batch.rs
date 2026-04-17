#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::cli::ParseCli;
use crate::model::{self, DocumentJson, ParseConfig, RunStatus};
use crate::parse_document::{build_document_json, eprint_partial_success, write_json_document};
use pdfium_render::prelude::*;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_parse_batch(
    pdfium: &Pdfium,
    cli: &ParseCli,
    inputs: &[PathBuf],
    page_filter: Option<&BTreeSet<u16>>,
    cfg: &ParseConfig,
) -> i32 {
    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut partial = 0usize;
    if let Some(dir) = &cli.output_dir {
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!("{}: {e}", dir.display());
            return 2;
        }
        for input in inputs {
            let Some(stem) = input.file_stem().and_then(|s| s.to_str()) else {
                eprintln!("bad filename: {}", input.display());
                failed += 1;
                continue;
            };
            let md_path = dir.join(format!("{stem}.md"));
            let json_path = cli.json.as_ref().map(|_| dir.join(format!("{stem}.json")));
            let debug_path = cli
                .debug_json
                .as_ref()
                .map(|_| dir.join(format!("{stem}.debug.json")));
            let batch_paths = BatchArtifacts {
                markdown: md_path.as_path(),
                json: json_path.as_deref(),
                debug: debug_path.as_deref(),
            };
            let r = batch_process_one(pdfium, cli, input, page_filter, cfg, &batch_paths);
            batch_accumulate(&mut ok, &mut failed, &mut partial, input, cli.quiet, r);
        }
    } else {
        for input in inputs {
            let md_path = input.with_extension("md");
            let batch_paths = BatchArtifacts {
                markdown: md_path.as_path(),
                json: None,
                debug: None,
            };
            let r = batch_process_one(pdfium, cli, input, page_filter, cfg, &batch_paths);
            batch_accumulate(&mut ok, &mut failed, &mut partial, input, cli.quiet, r);
        }
    }
    batch_exit_code(ok, partial, failed)
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
    artifacts: &BatchArtifacts<'_>,
    md: &str,
    outcome: BatchOutcome,
) -> BatchLine {
    if artifacts.markdown.exists() {
        eprintln!("refusing to overwrite {}", artifacts.markdown.display());
        return BatchLine::Skip;
    }
    let md_out = model::normalize_text(&model::postprocess_extracted_markdown(md));
    if fs::write(artifacts.markdown, md_out.as_bytes()).is_err() {
        if !quiet {
            eprintln!("{}: write error", input.display());
        }
        return BatchLine::Skip;
    }
    BatchLine::Outcome(outcome)
}

pub fn batch_process_one(
    pdfium: &Pdfium,
    cli: &ParseCli,
    input: &Path,
    page_filter: Option<&BTreeSet<u16>>,
    cfg: &ParseConfig,
    artifacts: &BatchArtifacts<'_>,
) -> BatchLine {
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
    let line = write_markdown_artifact(input, cli.quiet, artifacts, &md, outcome);
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
) -> Result<(), BatchLine> {
    let Some(path) = path else {
        return Ok(());
    };
    match write_json_document(path, dj) {
        Ok(()) => Ok(()),
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
    write_json_sidecar_if_present(artifacts.json, dj, quiet)?;
    write_json_sidecar_if_present(artifacts.debug, dj, quiet)
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
    use super::{batch_exit_code, status_outcome, BatchOutcome};
    use crate::model::RunStatus;

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
}
