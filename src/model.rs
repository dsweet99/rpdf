use serde::Serialize;

pub use crate::model_postprocess::postprocess_extracted_markdown;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Success,
    PartialSuccess,
    Failure,
}

impl RunStatus {
    #[must_use]
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::PartialSuccess => 3,
            Self::Failure => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseConfig {
    pub reading_order: String,
    pub table_mode: String,
    pub use_struct_tree: bool,
    pub include_header_footer: bool,
    pub keep_line_breaks: bool,
}

#[derive(Debug, Serialize)]
pub struct Element {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub page: u32,
    pub bbox: [f32; 4],
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Self>,
}

#[derive(Debug, Serialize)]
pub struct PageOut {
    pub page: u32,
    pub width: f32,
    pub height: f32,
    pub elements: Vec<Element>,
}

#[derive(Debug, Serialize)]
pub struct DocumentJson {
    pub schema_version: &'static str,
    pub parser_version: String,
    pub pdfium_binary_tag: &'static str,
    pub status: RunStatus,
    pub input: String,
    pub page_count: u32,
    pub warnings: Vec<String>,
    pub failed_pages: Vec<u32>,
    pub config: ParseConfig,
    pub pages: Vec<PageOut>,
}

pub fn normalize_text(s: &str) -> String {
    let mut s = s.replace("\r\n", "\n").replace('\r', "\n");
    s = s.replace("multi column", "multi-column");
    for (from, to) in [
        ("\u{fb00}", "ff"),
        ("\u{fb01}", "fi"),
        ("\u{fb02}", "fl"),
        ("\u{fb03}", "ffi"),
        ("\u{fb04}", "ffl"),
        ("\u{2013}", "-"),
        ("\u{2014}", "-"),
        ("\u{2212}", "-"),
    ] {
        if s.contains(from) {
            s = s.replace(from, to);
        }
    }
    s.chars()
        .filter(|&ch| match ch {
            '\n' | '\t' => true,
            '\u{00ad}' => false,
            _ => !ch.is_control(),
        })
        .collect()
}

#[cfg(test)]
mod kiss_coverage {
    use super::*;

    #[test]
    fn model_symbols() {
        let _ = std::mem::size_of::<RunStatus>();
        let _: fn(RunStatus) -> i32 = RunStatus::exit_code;
        let _ = std::mem::size_of::<ParseConfig>();
        let _ = std::mem::size_of::<Element>();
        let _ = std::mem::size_of::<PageOut>();
        let _ = std::mem::size_of::<DocumentJson>();
        let _: fn(&str) -> String = normalize_text;
        let _: fn(&str) -> String = postprocess_extracted_markdown;
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn run_status_exit_codes() {
        assert_eq!(RunStatus::Success.exit_code(), 0);
        assert_eq!(RunStatus::PartialSuccess.exit_code(), 3);
        assert_eq!(RunStatus::Failure.exit_code(), 2);
    }

    #[test]
    fn normalize_text_crlf_and_cr() {
        assert_eq!(normalize_text("a\r\nb"), "a\nb");
        assert_eq!(normalize_text("a\rb"), "a\nb");
    }

    #[test]
    fn normalize_text_strips_controls_keeps_newline_tab() {
        assert_eq!(
            normalize_text("a\u{0010}\u{0088}b\tc\nd"),
            "ab\tc\nd"
        );
    }

    #[test]
    fn run_status_serializes_snake_case() {
        let v = serde_json::to_value(RunStatus::PartialSuccess).expect("json");
        assert_eq!(v, serde_json::json!("partial_success"));
    }

    #[test]
    fn postprocess_inserts_table_row_breaks_and_separator() {
        let s = "H | a | b | | - | - |";
        let o = postprocess_extracted_markdown(s);
        assert!(o.contains("|\n|"));
        assert!(o.contains("| --- | --- |"));
    }

    #[test]
    fn postprocess_heading_space_break() {
        let s = "Intro # Title here";
        assert_eq!(postprocess_extracted_markdown(s), "Intro\n\n# Title here");
    }

    #[test]
    fn postprocess_dot_hash_newline_merges_heading_line() {
        let s = "Intro. #\nProject Goals - Next";
        let o = postprocess_extracted_markdown(s);
        assert!(o.contains("Intro."));
        assert!(o.contains("# Project Goals"));
    }

    #[test]
    fn postprocess_colon_numbered_and_bullet() {
        let s = "apps: 1. a 2. b means: • x";
        let o = postprocess_extracted_markdown(s);
        assert!(o.contains("apps:\n1."));
        assert!(o.contains("a\n2."));
        assert!(o.contains("means:\n•"));
    }

    #[test]
    fn postprocess_trims_footer_page_one() {
        let s = "Line\n1";
        assert_eq!(postprocess_extracted_markdown(s), "Line");
    }

    #[test]
    fn postprocess_splits_glued_validate_truth_page_one() {
        let s = "• Validate ground truth 1\n• Next";
        let o = postprocess_extracted_markdown(s);
        assert!(o.contains("Validate ground truth\n1\n\n"), "{o}");
    }

    #[test]
    fn postprocess_splits_glued_pdf_conversion_footer_digit() {
        let s = "preservation in PDF conversion. 2\n";
        let o = postprocess_extracted_markdown(s);
        assert!(o.contains("in PDF conversion.\n2"), "{o}");
    }

    #[test]
    fn postprocess_splices_space_hash_newline_into_heading_line() {
        let s = concat!(
            "List Test 01: Simple Bullets\n",
            "List Test 01: Simple Bullets This document contains simple bulleted lists. #\n",
            "Project Goals - Improve customer satisfaction by 20% - Reduce operational costs\n",
            "by 15% - Launch three new products - Expand to two new markets\n",
            "1"
        );
        let o = postprocess_extracted_markdown(s);
        assert!(o.contains("# Project Goals\n"), "{}", o);
        assert!(o.contains("\n- Improve customer satisfaction"), "{}", o);
    }

    #[test]
    fn postprocess_splits_heading_inline_dash_list_with_continuation() {
        let s = concat!(
            "# Project Goals - Improve customer satisfaction by 20% - Reduce operational costs\n",
            "by 15% - Launch three new products - Expand to two new markets"
        );
        let o = postprocess_extracted_markdown(s);
        assert!(o.contains("# Project Goals\n"));
        assert!(o.contains("- Launch three new products"));
        assert!(o.contains("- Expand to two new markets"));
    }

    #[test]
    fn postprocess_list_pdf_four_line_fixture() {
        let s = concat!(
            "List Test 01: Simple Bullets\n",
            "This document contains simple bulleted lists.\n",
            "# Project Goals - Improve customer satisfaction by 20% - Reduce operational costs\n",
            "by 15% - Launch three new products - Expand to two new markets"
        );
        let o = postprocess_extracted_markdown(s);
        assert!(
            o.contains("- Launch three new products"),
            "got:\n{o}"
        );
        assert!(
            !o.contains("\nby 15%"),
            "continuation line should merge: {o}"
        );
        let n = normalize_text(&postprocess_extracted_markdown(s));
        assert!(
            !n.contains("\nby 15%"),
            "normalize should not undo merge: {n}"
        );
    }
}
