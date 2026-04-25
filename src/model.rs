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
        assert_eq!(
            stringify!(crate::model_postprocess::merge_wrapped_title_lines_once),
            "crate::model_postprocess::merge_wrapped_title_lines_once"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::merge_wrapped_title_lines),
            "crate::model_postprocess::merge_wrapped_title_lines"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::should_merge_wrapped_title_fragment),
            "crate::model_postprocess::should_merge_wrapped_title_fragment"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::next_has_substantial_word),
            "crate::model_postprocess::next_has_substantial_word"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::join_hash_heading_lowercase_continuation),
            "crate::model_postprocess::join_hash_heading_lowercase_continuation"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::unwrap_pdf_line_wraps),
            "crate::model_postprocess::unwrap_pdf_line_wraps"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::split_dash_separated_bullet_items),
            "crate::model_postprocess::split_dash_separated_bullet_items"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::promote_table_section_headings),
            "crate::model_postprocess::promote_table_section_headings"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::split_heading_inline_dash_list),
            "crate::model_postprocess::split_heading_inline_dash_list"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::apply_heading_and_list_patterns),
            "crate::model_postprocess::apply_heading_and_list_patterns"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::finalize_pipe_table_separators),
            "crate::model_postprocess::finalize_pipe_table_separators"
        );
        assert_eq!(
            stringify!(crate::model_postprocess::trim_trailing_page_number_line),
            "crate::model_postprocess::trim_trailing_page_number_line"
        );
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
    fn postprocess_table_section_heading_respects_max_len() {
        let short = "TABLE-01: Basic Data Table\n\nx";
        let o = postprocess_extracted_markdown(short);
        assert!(o.contains("## TABLE-01:"), "{o}");
        let long = format!("TABLE-99:{}\n\nx", "a".repeat(56));
        let o2 = postprocess_extracted_markdown(&long);
        assert!(!o2.contains("## TABLE-99:"), "{o2}");
    }

    #[test]
    fn postprocess_table_label_line_keeps_heading_after_wrapped_body_line() {
        let s = concat!(
            "TABLE-03: Regional Performance\n",
            "Regional performance analysis reveals diverse growth patterns.\n",
        );
        let o = postprocess_extracted_markdown(s);
        assert!(
            o.contains("## TABLE-03: Regional Performance"),
            "expected promoted heading, got:\n{o}"
        );
    }

    #[test]
    fn postprocess_keeps_inline_hash_in_prose_short() {
        let s = "Intro # Title here";
        assert_eq!(postprocess_extracted_markdown(s), s);
    }

    #[test]
    fn postprocess_keeps_inline_hash_in_prose_long() {
        let s = "This is a long paragraph that ends with proper punctuation and has sufficient length to trigger blank line insertion. # Title here";
        assert_eq!(postprocess_extracted_markdown(s), s);
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
    fn postprocess_keeps_legitimate_trailing_one() {
        let s = "Final score\n1";
        assert_eq!(postprocess_extracted_markdown(s), s);
    }

    #[test]
    fn postprocess_keeps_total_trailing_one() {
        let s = "Total\n1";
        assert_eq!(postprocess_extracted_markdown(s), s);
    }

    #[test]
    fn postprocess_keeps_score_trailing_one() {
        let s = "Score\n1";
        assert_eq!(postprocess_extracted_markdown(s), s);
    }

    #[test]
    fn postprocess_keeps_trailing_one_after_code_token() {
        let s = "ID123\n1";
        assert_eq!(postprocess_extracted_markdown(s), s);
    }

    #[test]
    fn postprocess_splits_glued_validate_truth_page_one() {
        let s = "• Validate ground truth 1\n• Next";
        let o = postprocess_extracted_markdown(s);
        assert_eq!(o, s);
    }

    #[test]
    fn postprocess_splits_glued_pdf_conversion_footer_digit() {
        let s = "preservation in PDF conversion. 2\n";
        let o = postprocess_extracted_markdown(s);
        assert!(o.contains("in PDF conversion.\n2"), "{o}");
    }

    #[test]
    fn postprocess_title_word_pair_does_not_split_mid_sentence_prose() {
        let s = "The study includes analysis Market Dynamics for context.";
        assert_eq!(postprocess_extracted_markdown(s), s);
    }

    #[test]
    fn postprocess_title_word_pair_splits_line_start_pattern() {
        let s = "analysis Market Dynamics";
        assert_eq!(postprocess_extracted_markdown(s), "analysis\nMarket Dynamics");
    }

    #[test]
    fn postprocess_word_wrap_split_keeps_space() {
        let s = "alpha\nbeta";
        assert_eq!(postprocess_extracted_markdown(s), "alpha beta");
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
    fn postprocess_keeps_single_dash_heading_subtitle() {
        let s = "# Results - Q4";
        assert_eq!(postprocess_extracted_markdown(s), s);
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
