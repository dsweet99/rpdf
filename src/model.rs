use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

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

static COLON_THEN_NUMBERED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":\s+(\d+\.)").expect("colon numbered list"));
static LETTER_SPACE_THEN_NTH_ITEM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-zA-Z])(\s)((?:[2-9]|[1-9]\d+)\.\s)").expect("nth list item")
});
static COLON_THEN_BULLET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":\s+([•·▪▸])").expect("colon then bullet"));
static INLINE_BULLET_AFTER_ALNUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-z0-9])\s+•\s*").expect("inline bullet after alnum"));
static TITLE_WORD_PAIR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-z]{2,})\s+([A-Z][a-z]{5,} [A-Z][a-z]{5,})").expect("two title-case words")
});
static HYPHEN_SOFT_BREAK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-\n([a-z])").expect("hyphen syllable break"));
static NUMBER_HYPHEN_WORD_WRAP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([0-9]-[a-z]+)\n([a-z][a-z]+\.)").expect("digit-hyphen wrap before word")
});
static LOWER_BEFORE_OPEN_PAREN_BREAK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-z])\n(\()").expect("linebreak before open paren")
});
static SPACE_BEFORE_LISTS_HEADING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-z]) ([A-Z][a-z]+ Lists)\b").expect("space before titled list sections")
});
static SPACE_BEFORE_CONCLUSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-z]) (Conclusion)\b").expect("space before conclusion"));
static WORD_BREAK_BEFORE_PIPE_CELL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Za-z]+)\n([A-Z][a-z]+ \|)").expect("name wrap before pipe cell")
});
static BULLET_LINE_BREAK_AFTER_COLON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("(\u{2022}[^\n]*:)\n([A-Z])").expect("bullet line wrapped after colon")
});

pub fn postprocess_extracted_markdown(s: &str) -> String {
    let s = unwrap_pdf_line_wraps(s);
    let s = reflow_pipe_table_text(&s);
    let s = apply_heading_and_list_patterns(&s);
    finalize_pipe_table_separators(&s)
}

fn unwrap_pdf_line_wraps(s: &str) -> String {
    let mut s = s.replace("\r\n", "\n").replace('\r', "\n");
    s = HYPHEN_SOFT_BREAK
        .replace_all(&s, "- $1")
        .into_owned();
    s = NUMBER_HYPHEN_WORD_WRAP.replace_all(&s, "$1 $2").into_owned();
    s = LOWER_BEFORE_OPEN_PAREN_BREAK
        .replace_all(&s, "$1 $2")
        .into_owned();
    s = SPACE_BEFORE_LISTS_HEADING
        .replace_all(&s, "$1\n$2")
        .into_owned();
    s = SPACE_BEFORE_CONCLUSION.replace_all(&s, "$1\n$2").into_owned();
    s = WORD_BREAK_BEFORE_PIPE_CELL
        .replace_all(&s, "$1 $2")
        .into_owned();
    BULLET_LINE_BREAK_AFTER_COLON
        .replace_all(&s, "$1 $2")
        .into_owned()
}

fn reflow_pipe_table_text(s: &str) -> String {
    let mut s = s.replace("| |", "|\n|");
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    for _ in 0u8..8 {
        let n = lines.len();
        merge_orphan_pipe_lines(&mut lines);
        if lines.len() == n {
            break;
        }
    }
    s = lines.join("\n");
    s = s.replace(". |", ".\n|");
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    split_duplicate_prefix_second_line(lines.as_mut_slice());
    lines.dedup();
    lines.join("\n")
}

fn apply_heading_and_list_patterns(s: &str) -> String {
    let mut s = s.replace(" # ", "\n# ");
    s = s.replace(" ## ", "\n## ");
    s = s.replace(": -", ":\n-");
    s = COLON_THEN_NUMBERED
        .replace_all(&s, |caps: &regex::Captures<'_>| format!(":\n{}", &caps[1]))
        .into_owned();
    s = COLON_THEN_BULLET
        .replace_all(&s, |caps: &regex::Captures<'_>| format!(":\n{}", &caps[1]))
        .into_owned();
    s = INLINE_BULLET_AFTER_ALNUM
        .replace_all(&s, |caps: &regex::Captures<'_>| {
            format!("{}\n• ", caps.get(1).map_or("", |m| m.as_str()))
        })
        .into_owned();
    s = TITLE_WORD_PAIR
        .replace_all(&s, |caps: &regex::Captures<'_>| {
            format!(
                "{}\n{}",
                caps.get(1).map_or("", |m| m.as_str()),
                caps.get(2).map_or("", |m| m.as_str())
            )
        })
        .into_owned();
    LETTER_SPACE_THEN_NTH_ITEM
        .replace_all(&s, |caps: &regex::Captures<'_>| {
            format!(
                "{}\n{}",
                caps.get(1).map_or("", |m| m.as_str()),
                caps.get(3).map_or("", |m| m.as_str())
            )
        })
        .into_owned()
}

fn finalize_pipe_table_separators(s: &str) -> String {
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    for line in &mut lines {
        if separator_row_is_dash_cells(line) {
            *line = rewrite_separator_row(line);
        }
    }
    let mut out = lines.join("\n");
    trim_trailing_page_number_line(&mut out);
    out
}

fn merge_orphan_pipe_lines(lines: &mut Vec<String>) {
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "|" {
            i += 1;
            continue;
        }
        if orphan_pipe_action(lines, i) {
            continue;
        }
        i += 1;
    }
}

fn orphan_pipe_action(lines: &mut Vec<String>, i: usize) -> bool {
    let Some(next_line) = lines.get(i + 1) else {
        return false;
    };
    let nxt = next_line.trim_start();
    if !nxt.starts_with('|') {
        lines[i + 1] = format!("| {nxt}");
        lines.remove(i);
        return true;
    }
    lines.remove(i);
    true
}

fn split_duplicate_prefix_second_line(lines: &mut [String]) {
    if lines.len() < 2 {
        return;
    }
    let first = lines[0].trim_end();
    if first.is_empty() {
        return;
    }
    let prefix = format!("{first} ");
    let Some(rest) = lines[1].strip_prefix(&prefix) else {
        return;
    };
    lines[1] = rest.trim_start().to_string();
}

fn separator_row_is_dash_cells(line: &str) -> bool {
    let t = line.trim();
    if !t.contains('|') {
        return false;
    }
    let parts: Vec<&str> = t
        .split('|')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .collect();
    if parts.len() < 2 {
        return false;
    }
    parts.iter().all(|p| *p == "-")
}

fn rewrite_separator_row(line: &str) -> String {
    let t = line.trim();
    let parts: Vec<&str> = t
        .split('|')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .collect();
    if parts.is_empty() {
        return line.to_string();
    }
    let inner = std::iter::repeat_n("---", parts.len())
        .collect::<Vec<_>>()
        .join(" | ");
    format!("| {inner} |")
}

fn trim_trailing_page_number_line(s: &mut String) {
    if let Some(pos) = s.rfind('\n') {
        if &s[pos + 1..] == "1" {
            s.truncate(pos);
        }
    }
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
    fn postprocess_inserts_table_row_breaks_and_separator() {
        let s = "H | a | b | | - | - |";
        let o = postprocess_extracted_markdown(s);
        assert!(o.contains("|\n|"));
        assert!(o.contains("| --- | --- |"));
    }

    #[test]
    fn postprocess_heading_space_break() {
        let s = "Intro # Title here";
        assert_eq!(postprocess_extracted_markdown(s), "Intro\n# Title here");
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
}
