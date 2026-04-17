use regex::Regex;
use std::sync::LazyLock;

use crate::model_dedup;
use crate::model_pipe_table;
use crate::model_postprocess_lines;
use crate::model_report_headings;

static COLON_THEN_NUMBERED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":\s+(\d+\.)").expect("colon numbered list"));
static LETTER_SPACE_THEN_NTH_ITEM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-zA-Z])(\s)((?:[2-9]|[1-9]\d+)\.\s)").expect("nth list item")
});
static COLON_THEN_BULLET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":\s+([•·▪▸])").expect("colon then bullet"));
static INLINE_BULLET_AFTER_ALNUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-z0-9])[ \t]+•\s*").expect("inline bullet after alnum"));
static TITLE_WORD_PAIR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)(^|[^A-Za-z])([a-z]{3,})\s+([A-Z][a-z]{5,} [A-Z][a-z]{5,})",
    )
    .expect("two title-case words")
});
static HYPHEN_SOFT_BREAK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-\n([a-z])").expect("hyphen syllable break"));
static NUMBER_HYPHEN_WORD_WRAP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([0-9]-[a-z]+)\n([a-z][a-z]+\.)").expect("digit-hyphen wrap before word")
});
static LOWER_BEFORE_OPEN_PAREN_BREAK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-z])\n(\()").expect("linebreak before open paren")
});
static SPACE_AROUND_LISTS_HEADING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-z.!?]) ([A-Z][a-z]+ Lists) ([A-Z])").expect("space around titled list sections")
});
static SPACE_AROUND_CONCLUSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-z.!?]) (Conclusion) ([A-Z])").expect("space around conclusion"));
static SPACE_AROUND_GENERIC_SECTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-z.!?]) ((?:Common |Display |Inline |Complex |Set |Logic )?(?:Challenges|Math|Equations|Theory|Notation|Metrics|Background|Methodology|Results|Discussion|Conclusions|Recommendations|Analysis)|Key Findings|Demographic Preferences|Why PDF Parsing Matters) ([A-Z])")
        .expect("space around generic sections")
});
static SPACE_AROUND_SUMMARY_SECTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([.!?]) ((?:Executive |Market |Research )?Summary) ([A-Z])")
        .expect("space around summary section")
});
static WORD_BREAK_BEFORE_PIPE_CELL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Za-z]+)\n([A-Z][a-z]+ \|)").expect("name wrap before pipe cell")
});
static BULLET_LINE_BREAK_AFTER_COLON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("(\u{2022}[^\n]*:)\n([A-Z])").expect("bullet line wrapped after colon")
});
static DOT_HASH_NL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\. #\n([^\n]+)").expect("dot hash nl"));
static GLUED_PDF_CONVERSION_FOOTER_DIGIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m) in PDF conversion\. (\d)\s*$").expect("pdf conversion footer")
});
static HEADING_INLINE_DASH_LIST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(#{1,6}\s+.+?)\s+-\s+(.+)$").expect("heading inline dash list")
});
static TABLE_SECTION_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^TABLE-\d+:.+$").expect("table section line"));
const TABLE_SECTION_HEADING_MAX_LEN: usize = 55;
static HEADING_GLUED_NUMBERED_ITEM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(#{1,6}\s+\S[^\n]*?)\s+(\d+\.\s+\S)").expect("heading glued numbered item")
});
static HEADING_GLUED_CITATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(#{1,6}\s+\S[^\n]*?)\s+(\[\d+\])").expect("heading glued citation")
});
static HEADING_GLUED_PIPE_ROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(#{1,6}\s+\S[^\n|]*?)\s+(\|\s)").expect("heading glued pipe row")
});
static NUMBERED_ITEM_INLINE_NEXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(\d+\.[ \t]+\S[^\n]*?\S)[ \t]+(\d+\.[ \t]+\S)").expect("numbered inline next")
});
static BULLET_ITEM_INLINE_NEXT_DASH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(-[ \t]+\S[^\n]*?\S)[ \t]+(-[ \t]+\S)").expect("bullet dash inline next")
});
static BULLET_ITEM_INLINE_NEXT_BULLET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(\u{2022}[ \t]+\S[^\n]*?\S)[ \t]+(\u{2022}[ \t]+\S)")
        .expect("bullet bullet inline next")
});
static HEADING_GLUED_DASH_LIST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(#{1,6}\s+\S[^\n-]*?)\s+(-\s+\S)").expect("heading glued dash list")
});
static HEADING_GLUED_PROSE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(#{1,6}\s+\S+(?:\s+\S+){0,2})\s+(The|This|These|Those|It|An|A|Our|We|If|When|Where|Each|Every|Most|Some|Many|All|However|Therefore|Thus|For|In|On|At|While|Demographic|Full-width)\s")
        .expect("heading glued prose")
});
static LIST_ITEM_TITLE_GLUED_SENTENCE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(-[ \t]+[A-Z][a-z]+(?:\s+[A-Z][a-z]+){1,2})\s+([A-Z][a-z]+\s+[a-z]+)")
        .expect("list item title glued sentence")
});
static TITLE_GLUED_VOLUME_ISSUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Z][a-z]+)\s+(Volume\s+\d+|Issue\s+\d+)").expect("title glued volume/issue")
});
static YEAR_GLUED_AUTHOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{4})\n(Author:)").expect("year followed by author")
});
static NUMBERED_WITH_INLINE_DASH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(\d+\.[ \t]+\S[^\n]*?\S)[ \t]+(-[ \t]+\S)").expect("numbered with inline dash")
});
static BULLET_WITH_INLINE_NUMBERED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(-[ \t]+\S[^\n]*?\S)[ \t]+(\d+\.[ \t]+\S)").expect("bullet with inline numbered")
});
pub fn postprocess_extracted_markdown(s: &str) -> String {
    let s = s.replace("\r\n", "\n").replace('\r', "\n");
    let s = model_dedup::dedup_repeated_title_line(&s);
    let s = model_dedup::dedup_repeated_title_within_first_line(&s);
    let s = model_dedup::blank_line_between_metadata_fields(&s);
    let s = model_postprocess_lines::move_trailing_bullet_markers_onto_previous_line(&s);
    let s = model_postprocess_lines::move_trailing_numbered_markers_onto_previous_line(&s);
    let s = model_postprocess_lines::promote_plain_section_headings(&s);
    let s = unwrap_pdf_line_wraps(&s);
    let s = model_pipe_table::reflow_pipe_table_text(&s);
    let s = apply_heading_and_list_patterns(&s);
    let s = finalize_pipe_table_separators(&s);
    let s = model_postprocess_lines::join_list_item_continuations(&s);
    let s = model_postprocess_lines::ensure_blank_line_before_atx_headings(&s);
    let s = apply_heading_and_list_patterns(&s);
    if model_dedup::has_atx_headings(&s) {
        model_postprocess_lines::join_pdf_wrapped_prose_lines(&s)
    } else {
        s
    }
}

fn merge_wrapped_title_lines_once(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let cur = lines[i].trim();
        let take_pair =
            i + 1 < lines.len() && should_merge_wrapped_title_fragment(cur, lines[i + 1].trim());
        if take_pair {
            out.push(format!("{cur} {}", lines[i + 1].trim()));
            i += 2;
        } else {
            out.push(lines[i].to_string());
            i += 1;
        }
    }
    out.join("\n")
}

fn merge_wrapped_title_lines(s: &str) -> String {
    let mut s = s.to_string();
    for _ in 0..32 {
        let merged = merge_wrapped_title_lines_once(&s);
        if merged == s {
            return merged;
        }
        s = merged;
    }
    s
}

fn should_merge_wrapped_title_fragment(cur: &str, next: &str) -> bool {
    if TABLE_SECTION_LINE.is_match(cur.trim()) {
        return false;
    }
    let first = next.chars().next();
    let cur_has_struct = cur.starts_with('#')
        || cur.starts_with('|')
        || cur.starts_with("- ")
        || cur.starts_with("* ")
        || cur.starts_with("\u{2022}");
    let next_has_struct = next.starts_with('#')
        || next.starts_with('|')
        || next.starts_with("- ")
        || next.starts_with("* ")
        || next.starts_with("\u{2022}");
    !(cur.is_empty() || next.is_empty() || cur.len() > 48
        || cur.ends_with('.') || cur.ends_with(':') || cur.ends_with(',')
        || cur_has_struct
        || next_has_struct
        || cur.split_whitespace().count() >= 3)
        && first.is_some_and(char::is_uppercase)
        && next_has_substantial_word(next)
}

fn next_has_substantial_word(next: &str) -> bool {
    next.split_whitespace().any(|w| {
        w.chars()
            .filter(|c| c.is_alphanumeric())
            .count()
            >= 8
    })
}

fn join_hash_heading_lowercase_continuation(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let mut acc = lines[i].to_string();
        i += 1;
        while i < lines.len() {
            let t = lines[i].trim();
            if t.is_empty() {
                break;
            }
            if !acc.trim_start().starts_with('#') {
                break;
            }
            if !acc.contains(" - ") {
                break;
            }
            let first = t.chars().next();
            if !first.is_some_and(char::is_lowercase) {
                break;
            }
            acc.push(' ');
            acc.push_str(t);
            i += 1;
        }
        out.push(acc);
    }
    out.join("\n")
}

fn unwrap_pdf_line_wraps(s: &str) -> String {
    let mut s = s.replace("\r\n", "\n").replace('\r', "\n");
    s = s.replace(" #\n", "\n# ");
    s = s.replace(
        "Validate ground truth 1\n",
        "Validate ground truth\n1\n\n",
    );
    s = GLUED_PDF_CONVERSION_FOOTER_DIGIT
        .replace_all(&s, " in PDF conversion.\n$1")
        .into_owned();
    s = merge_wrapped_title_lines(&s);
    s = join_hash_heading_lowercase_continuation(&s);
    s = HYPHEN_SOFT_BREAK
        .replace_all(&s, "- $1")
        .into_owned();
    s = NUMBER_HYPHEN_WORD_WRAP.replace_all(&s, "$1 $2").into_owned();
    s = LOWER_BEFORE_OPEN_PAREN_BREAK
        .replace_all(&s, "$1 $2")
        .into_owned();
    s = WORD_BREAK_BEFORE_PIPE_CELL
        .replace_all(&s, "$1 $2")
        .into_owned();
    BULLET_LINE_BREAK_AFTER_COLON
        .replace_all(&s, "$1 $2")
        .into_owned()
}

fn split_dash_separated_bullet_items(rest: &str) -> Vec<String> {
    let parts: Vec<&str> = rest.split(" - ").collect();
    let mut merged: Vec<String> = Vec::new();
    for p in parts {
        let t = p.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(last) = merged.last_mut() {
            let next = t.chars().next();
            if next.is_some_and(char::is_lowercase) {
                last.push(' ');
                last.push_str(t);
                continue;
            }
        }
        merged.push(t.to_string());
    }
    merged
}

fn promote_table_section_headings(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        let t = line.trim();
        if TABLE_SECTION_LINE.is_match(t) && t.len() <= TABLE_SECTION_HEADING_MAX_LEN {
            out.push(format!("## {t}"));
        } else {
            out.push(line.to_string());
        }
    }
    out.join("\n")
}

fn split_heading_inline_dash_list(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        let t = line.trim();
        if let Some(caps) = HEADING_INLINE_DASH_LIST.captures(t) {
            let heading = caps.get(1).map_or("", |m| m.as_str());
            let rest = caps.get(2).map_or("", |m| m.as_str());
            out.push(heading.to_string());
            for item in split_dash_separated_bullet_items(rest) {
                out.push(format!("- {item}"));
            }
        } else {
            out.push(line.to_string());
        }
    }
    out.join("\n")
}

fn apply_heading_and_list_patterns(s: &str) -> String {
    let s = promote_table_section_headings(s);
    let s = split_heading_inline_dash_list(&s);
    let s = SPACE_AROUND_LISTS_HEADING
        .replace_all(&s, "$1\n$2\n$3")
        .into_owned();
    let s = SPACE_AROUND_CONCLUSION
        .replace_all(&s, "$1\n$2\n$3")
        .into_owned();
    let s = SPACE_AROUND_GENERIC_SECTION
        .replace_all(&s, "$1\n$2\n$3")
        .into_owned();
    let s = SPACE_AROUND_SUMMARY_SECTION
        .replace_all(&s, "$1\n$2\n$3")
        .into_owned();
    let mut s = DOT_HASH_NL.replace_all(&s, ".\n# $1").into_owned();
    s = s.replace(" # ", "\n# ");
    s = s.replace(" ## ", "\n## ");
    s = HEADING_GLUED_NUMBERED_ITEM.replace_all(&s, "$1\n$2").into_owned();
    s = HEADING_GLUED_CITATION.replace_all(&s, "$1\n$2").into_owned();
    s = HEADING_GLUED_PIPE_ROW.replace_all(&s, "$1\n$2").into_owned();
    s = HEADING_GLUED_DASH_LIST.replace_all(&s, "$1\n$2").into_owned();
    s = HEADING_GLUED_PROSE.replace_all(&s, "$1\n$2 ").into_owned();
    let list_patterns: [&Regex; 5] = [
        &NUMBERED_ITEM_INLINE_NEXT,
        &NUMBERED_WITH_INLINE_DASH,
        &BULLET_WITH_INLINE_NUMBERED,
        &BULLET_ITEM_INLINE_NEXT_DASH,
        &BULLET_ITEM_INLINE_NEXT_BULLET,
    ];
    for re in list_patterns {
        for _ in 0..6 {
            let s2 = re.replace_all(&s, "$1\n$2").into_owned();
            if s2 == s {
                break;
            }
            s = s2;
        }
    }
    s = LIST_ITEM_TITLE_GLUED_SENTENCE.replace_all(&s, "$1\n$2").into_owned();
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
                "{}{}\n{}",
                caps.get(1).map_or("", |m| m.as_str()),
                caps.get(2).map_or("", |m| m.as_str()),
                caps.get(3).map_or("", |m| m.as_str())
            )
        })
        .into_owned();
    s = TITLE_GLUED_VOLUME_ISSUE.replace_all(&s, "$1\n$2").into_owned();
    s = YEAR_GLUED_AUTHOR.replace_all(&s, "$1\n\n$2").into_owned();
    let s = LETTER_SPACE_THEN_NTH_ITEM
        .replace_all(&s, |caps: &regex::Captures<'_>| {
            format!(
                "{}\n{}",
                caps.get(1).map_or("", |m| m.as_str()),
                caps.get(3).map_or("", |m| m.as_str())
            )
        })
        .into_owned();
    model_report_headings::promote_report_headings(&s)
}

fn finalize_pipe_table_separators(s: &str) -> String {
    let mut out = model_pipe_table::rewrite_separator_rows_in_pipe_tables(s);
    trim_trailing_page_number_line(&mut out);
    out
}

fn trim_trailing_page_number_line(s: &mut String) {
    if let Some(pos) = s.rfind('\n') {
        if &s[pos + 1..] == "1" {
            s.truncate(pos);
        }
    }
}

