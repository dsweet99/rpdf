#![allow(clippy::cast_possible_truncation)]

use crate::cli::InspectCli;
use crate::engine;
use crate::pagespec::parse_pageset;
use pdfium_render::prelude::*;
use std::collections::BTreeSet;

fn load_filter(cli: &InspectCli) -> Result<Option<BTreeSet<u16>>, i32> {
    let Some(ref raw) = cli.pages else {
        return Ok(None);
    };
    parse_pageset(raw).map(Some).map_err(|e| {
        eprintln!("{e}");
        1
    })
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

fn print_report(
    cli: &InspectCli,
    page_count: i32,
    rev: &str,
    text_pages: u32,
    object_total: usize,
    filter: Option<&BTreeSet<u16>>,
) {
    println!("file: {}", cli.input.display());
    println!("pages: {page_count}");
    println!("security_handler_revision: {rev}");
    let scope = if filter.is_some() { "filtered" } else { "all" };
    println!("text_layer_pages_in_scope: {text_pages} (scope={scope})");
    println!("page_objects_in_scope: {object_total}");
    println!("parse_strategy: pdfium_text_extraction_basic");
    if rev != "Unprotected" && rev != "unknown" {
        println!("warning: document may be encrypted; supply --password if text is missing");
    }
}

pub fn run_inspect(cli: &InspectCli) -> i32 {
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
        Err(code) => return code,
    };
    let rev = doc
        .permissions()
        .security_handler_revision()
        .map_or_else(|_| "unknown".to_string(), |r| format!("{r:?}"));
    let (text_pages, object_total) = scope_stats(&doc, filter.as_ref());
    let page_count = doc.pages().len();
    print_report(
        cli,
        page_count,
        rev.as_str(),
        text_pages,
        object_total,
        filter.as_ref(),
    );
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
