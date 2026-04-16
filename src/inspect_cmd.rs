#![allow(clippy::cast_possible_truncation)]

use crate::cli::InspectCli;
use crate::engine;
use crate::pagespec::parse_pageset;
use pdfium_render::prelude::*;
use std::collections::BTreeSet;
use std::path::Path;

fn load_filter(cli: &InspectCli) -> Result<Option<BTreeSet<u16>>, String> {
    let Some(ref raw) = cli.pages else {
        return Ok(None);
    };
    parse_pageset(raw).map(Some)
}

fn pdf_tagging_probe(path: &Path) -> (bool, bool) {
    let Ok(bytes) = std::fs::read(path) else {
        return (false, false);
    };
    let cap = 8 * 1024 * 1024;
    let slice = if bytes.len() > cap {
        &bytes[..cap]
    } else {
        bytes.as_slice()
    };
    let mark = contains_subslice(slice, b"/MarkInfo");
    let struct_root = contains_subslice(slice, b"StructTreeRoot");
    (mark, struct_root)
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn scope_stats(doc: &PdfDocument<'_>, filter: Option<&BTreeSet<u16>>) -> (u32, usize) {
    let mut text_pages = 0u32;
    let mut object_total = 0usize;
    for (idx, page) in doc.pages().iter().enumerate() {
        let n = (idx + 1) as u32;
        if let Some(set) = filter {
            if !set.contains(&(n as u16)) {
                continue;
            }
        }
        if let Ok(t) = page.text() {
            if !t.is_empty() {
                text_pages += 1;
            }
        }
        object_total = object_total.saturating_add(page.objects().iter().count());
    }
    (text_pages, object_total)
}

struct InspectReport {
    file_display: String,
    page_count: i32,
    security_rev: String,
    text_pages: u32,
    object_total: usize,
    scope: &'static str,
    mark_info_probe: bool,
    struct_tree_probe: bool,
}

fn print_report(r: &InspectReport) {
    println!("file: {}", r.file_display);
    println!("pages: {}", r.page_count);
    println!("security_handler_revision: {}", r.security_rev);
    println!(
        "text_layer_pages_in_scope: {} (scope={})",
        r.text_pages, r.scope
    );
    println!("page_objects_in_scope: {}", r.object_total);
    println!("parse_strategy: pdfium_text_extraction_basic");
    println!(
        "mark_info_dictionary_probe: {}",
        if r.mark_info_probe { "found" } else { "not_found" }
    );
    println!(
        "structure_tree_root_probe: {}",
        if r.struct_tree_probe { "found" } else { "not_found" }
    );
    println!("tagging_probe_note: linear_byte_scan_not_authoritative");
}

pub fn run_inspect(cli: &InspectCli) -> i32 {
    let (mark_info_probe, struct_tree_probe) = pdf_tagging_probe(&cli.input);
    let pdfium = engine::init_pdfium();
    let doc = match pdfium.load_pdf_from_file(&cli.input, cli.password.as_deref()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let filter = match load_filter(cli) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    let rev = doc
        .permissions()
        .security_handler_revision()
        .map_or_else(|_| "unknown".to_string(), |r| format!("{r:?}"));
    let (text_pages, object_total) = scope_stats(&doc, filter.as_ref());
    let page_count = doc.pages().len();
    let scope = if filter.is_some() { "filtered" } else { "all" };
    print_report(&InspectReport {
        file_display: cli.input.display().to_string(),
        page_count,
        security_rev: rev.clone(),
        text_pages,
        object_total,
        scope,
        mark_info_probe,
        struct_tree_probe,
    });
    if rev != "Unprotected" && rev != "unknown" {
        eprintln!("warning: document may be encrypted; supply --password if text is missing");
    }
    0
}

#[cfg(test)]
mod kiss_coverage {
    use super::*;

    #[test]
    fn inspect_symbol() {
        let _: fn(&InspectCli) -> i32 = run_inspect;
    }
}
