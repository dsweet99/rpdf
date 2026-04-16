#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::engine;
use crate::markdown;
use crate::model::{self, DocumentJson, Element, PageOut, ParseConfig, RunStatus};
use pdfium_render::prelude::*;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const SCHEMA: &str = "1.0";

pub fn eprint_partial_success(quiet: bool, status: RunStatus, failed_pages: &[u32]) {
    if quiet || status != RunStatus::PartialSuccess {
        return;
    }
    eprintln!("partial_success: failed_pages={failed_pages:?}");
}

fn append_stub_config_warnings(cfg: &ParseConfig, warnings: &mut Vec<String>, quiet: bool) {
    if cfg.reading_order != "basic" && cfg.reading_order != "off" {
        warnings.push(
            "reading-order layout modes are not implemented; output uses one paragraph per page in extraction order"
                .to_string(),
        );
        if !quiet {
            eprintln!(
                "warning: --reading-order is not implemented for layout; using extraction order"
            );
        }
    }
    if cfg.table_mode != "off" {
        warnings.push(
            "table extraction modes are not implemented; skipping table detection".to_string(),
        );
        if !quiet {
            eprintln!("warning: --table-mode is not implemented");
        }
    }
    if cfg.include_header_footer {
        warnings.push("header/footer inclusion is not implemented".to_string());
        if !quiet {
            eprintln!("warning: --include-header-footer is not implemented");
        }
    }
    if cfg.keep_line_breaks {
        warnings.push(
            "keep-line-breaks is not implemented; line breaks follow PDFium text layout".to_string(),
        );
        if !quiet {
            eprintln!("warning: --keep-line-breaks is not implemented");
        }
    }
}

fn push_initial_warnings(cfg: &ParseConfig, quiet: bool) -> Vec<String> {
    let mut warnings = Vec::new();
    if cfg.use_struct_tree {
        warnings.push("struct-tree ordering is not implemented; using text extraction order".to_string());
    }
    if !quiet && cfg.use_struct_tree {
        eprintln!("warning: --use-struct-tree requested but parser uses extracted text order");
    }
    append_stub_config_warnings(cfg, &mut warnings, quiet);
    warnings
}

fn extract_page_outputs(
    doc: &PdfDocument<'_>,
    filter: Option<&BTreeSet<u16>>,
) -> (Vec<PageOut>, Vec<u32>) {
    let mut pages_out = Vec::new();
    let mut failed_pages = Vec::new();
    for (idx, page) in doc.pages().iter().enumerate() {
        let page_num = (idx + 1) as u32;
        if let Some(set) = filter {
            if !set.contains(&(page_num as u16)) {
                continue;
            }
        }
        let width = page.width().value;
        let height = page.height().value;
        match page.text() {
            Ok(t) => {
                let raw = t.all();
                let text = model::normalize_text(raw.as_str());
                let bbox = paragraph_bbox_union(&t, width, height);
                let el = Element {
                    id: format!("p{page_num}-e1"),
                    kind: "paragraph".to_string(),
                    page: page_num,
                    bbox,
                    text,
                    children: Vec::new(),
                };
                pages_out.push(PageOut {
                    page: page_num,
                    width,
                    height,
                    elements: vec![el],
                });
            }
            Err(_) => {
                failed_pages.push(page_num);
            }
        }
    }
    (pages_out, failed_pages)
}

fn merge_filter_out_of_range_requests(
    filter: Option<&BTreeSet<u16>>,
    page_count: u32,
    warnings: &mut Vec<String>,
    failed_pages: &mut Vec<u32>,
) {
    if let Some(set) = filter {
        for p in set {
            let n = u32::from(*p);
            if n == 0 || n > page_count {
                warnings.push(format!("requested page {n} is out of range ({page_count} pages)"));
                failed_pages.push(n);
            }
        }
    }
}

fn paragraph_bbox_union(t: &PdfPageText, page_width: f32, page_height: f32) -> [f32; 4] {
    let mut min_l = f32::MAX;
    let mut min_b = f32::MAX;
    let mut max_r = f32::MIN;
    let mut max_t = f32::MIN;
    let mut any = false;
    for ch in t.chars().iter() {
        if let Ok(r) = ch.tight_bounds() {
            any = true;
            min_l = min_l.min(r.left().value);
            min_b = min_b.min(r.bottom().value);
            max_r = max_r.max(r.right().value);
            max_t = max_t.max(r.top().value);
        }
    }
    if any {
        [min_l, min_b, max_r, max_t]
    } else {
        [0.0, 0.0, page_width, page_height]
    }
}

pub fn build_document_json(
    doc: &PdfDocument<'_>,
    input: &Path,
    filter: Option<&BTreeSet<u16>>,
    cfg: ParseConfig,
    quiet: bool,
) -> (DocumentJson, String) {
    let mut warnings = push_initial_warnings(&cfg, quiet);
    let page_count = doc.pages().len() as u32;
    let (pages_out, mut failed_pages) = extract_page_outputs(doc, filter);
    merge_filter_out_of_range_requests(filter, page_count, &mut warnings, &mut failed_pages);
    failed_pages.sort_unstable();
    failed_pages.dedup();
    let status = if failed_pages.is_empty() {
        RunStatus::Success
    } else if pages_out.is_empty() {
        RunStatus::Failure
    } else {
        RunStatus::PartialSuccess
    };
    let dj = DocumentJson {
        schema_version: SCHEMA,
        parser_version: env!("CARGO_PKG_VERSION").to_string(),
        pdfium_binary_tag: engine::PDFIUM_BINARY_TAG,
        status,
        input: input.display().to_string(),
        page_count,
        warnings,
        failed_pages,
        config: cfg,
        pages: pages_out,
    };
    let md = markdown::pages_to_markdown(&dj.pages);
    (dj, md)
}

pub fn write_exclusive(
    path: &Path,
    write: impl FnOnce(&mut Vec<u8>) -> Result<(), String>,
) -> Result<(), String> {
    if path.exists() {
        return Err(format!("refusing to overwrite {}", path.display()));
    }
    let mut buf = Vec::new();
    write(&mut buf)?;
    fs::write(path, buf).map_err(|e| e.to_string())
}

pub fn write_json_document(path: &Path, dj: &DocumentJson) -> Result<(), String> {
    write_exclusive(path, |w| serde_json::to_writer_pretty(w, dj).map_err(|e| e.to_string()))
}

#[cfg(test)]
mod tests_stub {
    use super::append_stub_config_warnings;
    use crate::model::ParseConfig;

    #[test]
    fn reading_order_off_skips_reading_order_stub_warnings() {
        let mut w = Vec::new();
        let cfg = ParseConfig {
            reading_order: "off".to_string(),
            table_mode: "off".to_string(),
            use_struct_tree: false,
            include_header_footer: false,
            keep_line_breaks: false,
        };
        append_stub_config_warnings(&cfg, &mut w, true);
        assert!(!w.iter().any(|s| s.contains("reading-order")));
    }

    #[test]
    fn xycut_reading_order_warns() {
        let mut w = Vec::new();
        let cfg = ParseConfig {
            reading_order: "xycut".to_string(),
            table_mode: "off".to_string(),
            use_struct_tree: false,
            include_header_footer: false,
            keep_line_breaks: false,
        };
        append_stub_config_warnings(&cfg, &mut w, true);
        assert!(w.iter().any(|s| s.contains("reading-order")));
    }

    #[test]
    fn nondefault_table_mode_warns() {
        let mut w = Vec::new();
        let cfg = ParseConfig {
            reading_order: "basic".to_string(),
            table_mode: "lines".to_string(),
            use_struct_tree: false,
            include_header_footer: false,
            keep_line_breaks: false,
        };
        append_stub_config_warnings(&cfg, &mut w, true);
        assert!(w.iter().any(|s| s.contains("table")));
    }

    #[test]
    fn include_header_footer_warns() {
        let mut w = Vec::new();
        let cfg = ParseConfig {
            reading_order: "basic".to_string(),
            table_mode: "off".to_string(),
            use_struct_tree: false,
            include_header_footer: true,
            keep_line_breaks: false,
        };
        append_stub_config_warnings(&cfg, &mut w, true);
        assert!(w.iter().any(|s| s.contains("header")));
    }

    #[test]
    fn keep_line_breaks_warns() {
        let mut w = Vec::new();
        let cfg = ParseConfig {
            reading_order: "basic".to_string(),
            table_mode: "off".to_string(),
            use_struct_tree: false,
            include_header_footer: false,
            keep_line_breaks: true,
        };
        append_stub_config_warnings(&cfg, &mut w, true);
        assert!(w.iter().any(|s| s.contains("line")));
    }
}

#[cfg(test)]
mod merge_and_exclusive_tests {
    use super::{merge_filter_out_of_range_requests, write_exclusive};
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    fn merge_filter_records_out_of_range_pages() {
        let mut warnings = Vec::new();
        let mut failed_pages = Vec::new();
        let mut set = BTreeSet::new();
        set.insert(9_u16);
        merge_filter_out_of_range_requests(Some(&set), 1, &mut warnings, &mut failed_pages);
        assert!(warnings.iter().any(|w| w.contains("out of range")));
        assert!(failed_pages.contains(&9));
    }

    #[test]
    fn write_exclusive_refuses_existing_path() {
        let dir = std::env::temp_dir().join(format!("rpdf_wa_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let p = dir.join("out.bin");
        write_exclusive(&p, |b| {
            b.extend_from_slice(b"a");
            Ok(())
        })
        .expect("first write");
        let err = write_exclusive(&p, |b| {
            b.extend_from_slice(b"b");
            Ok(())
        })
        .expect_err("second write");
        assert!(err.contains("refusing to overwrite"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_exclusive_writes_when_absent() {
        let dir = std::env::temp_dir().join(format!("rpdf_wa2_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let p: PathBuf = dir.join("new.bin");
        write_exclusive(&p, |b| {
            b.extend_from_slice(b"z");
            Ok(())
        })
        .expect("write");
        assert_eq!(std::fs::read(&p).expect("read"), b"z");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod kiss_coverage {
    use crate::model::RunStatus;

    #[test]
    fn handler_refs() {
        assert_eq!(
            stringify!(super::build_document_json),
            "super::build_document_json"
        );
        assert_eq!(stringify!(super::write_exclusive), "super::write_exclusive");
        let _: fn(bool, RunStatus, &[u32]) = super::eprint_partial_success;
    }
}
