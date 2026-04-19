use regex::Regex;
use std::sync::LazyLock;

static ATX_HEADING_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#{1,6}\s").expect("atx heading"));
static NUMBERED_MARKER_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+\.$").expect("numbered marker"));
static NUMBERED_LIST_ITEM_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+\.\s").expect("numbered list item prefix"));
static SECTION_HEADING_LEVEL1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(Executive Summary|Introduction|Background|Methodology|Results|Conclusion|Recommendations|References|Discussion|Abstract|Summary|Overview|Acknowledgments)$",
    )
    .expect("section level1")
});
static SECTION_HEADING_LEVEL2: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(Findings)$").expect("section level2"));
static SECTION_HEADING_LEVEL3: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(Key Observations)$").expect("section level3"));
static STRUCT_LINE_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\s*)(#{1,6}\s|[-*]\s|\d{1,3}\.\s|\|[^|]|>|```|•\s)").expect("struct prefix")
});

fn is_atx_heading_line(line: &str) -> bool {
    ATX_HEADING_LINE.is_match(line.trim_start())
}

pub fn ensure_blank_line_before_atx_headings(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let mut out: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if is_atx_heading_line(line) && i > 0 {
            let prev = lines[i - 1].trim();
            let last_blank = out.last().is_some_and(|l| l.trim().is_empty());
            let long_sentence = matches!(prev.chars().last(), Some('.' | '!' | '?')) && prev.len() > 60;
            let is_metadata = prev.starts_with("Report ID:")
                || prev.starts_with("Date:")
                || prev.starts_with("Author:")
                || prev.starts_with("ID:");
            let needs_blank = !prev.is_empty()
                && (is_atx_heading_line(prev) || long_sentence || is_metadata);
            if !last_blank && needs_blank {
                out.push(String::new());
            }
        }
        out.push((*line).to_string());
    }
    out.join("\n")
}

pub fn move_trailing_bullet_markers_onto_previous_line(s: &str) -> String {
    move_trailing_marker_lines(s, |t| match t {
        "\u{2022}" | "\u{2022} " => Some("\u{2022}".to_string()),
        _ => None,
    }, |stripped| {
        !stripped.starts_with('\u{2022}') && !stripped.starts_with('|')
    })
}

pub fn move_trailing_numbered_markers_onto_previous_line(s: &str) -> String {
    move_trailing_marker_lines(s, |t| {
        if NUMBERED_MARKER_LINE.is_match(t) {
            Some(t.to_string())
        } else {
            None
        }
    }, |stripped| {
        !stripped.is_empty() && !STRUCT_LINE_PREFIX.is_match(stripped)
    })
}

fn move_trailing_marker_lines(
    s: &str,
    marker_of: impl Fn(&str) -> Option<String>,
    prev_ok: impl Fn(&str) -> bool,
) -> String {
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    let mut i = 0usize;
    while i < lines.len() {
        if try_move_marker(&mut lines, i, &marker_of, &prev_ok) {
            continue;
        }
        i += 1;
    }
    lines.join("\n")
}

fn try_move_marker(
    lines: &mut Vec<String>,
    i: usize,
    marker_of: &impl Fn(&str) -> Option<String>,
    prev_ok: &impl Fn(&str) -> bool,
) -> bool {
    let Some(marker) = marker_of(lines[i].trim()) else {
        return false;
    };
    let Some(prev_idx) = previous_text_line_index(lines, i) else {
        return false;
    };
    let prev = lines[prev_idx].clone();
    let stripped = prev.trim_start();
    if is_atx_heading_line(&prev) || !prev_ok(stripped) {
        return false;
    }
    lines[prev_idx] = format!("{marker} {stripped}");
    lines.remove(i);
    true
}

fn previous_text_line_index(lines: &[String], i: usize) -> Option<usize> {
    let mut j = i;
    while j > 0 {
        j -= 1;
        if !lines[j].trim().is_empty() {
            return Some(j);
        }
    }
    None
}

pub fn promote_plain_section_headings(s: &str) -> String {
    s.lines().map(promote_one_line).collect::<Vec<_>>().join("\n")
}

fn promote_one_line(line: &str) -> String {
    let t = line.trim();
    if SECTION_HEADING_LEVEL1.is_match(t) {
        format!("# {t}")
    } else if SECTION_HEADING_LEVEL2.is_match(t) {
        format!("## {t}")
    } else if SECTION_HEADING_LEVEL3.is_match(t) {
        format!("### {t}")
    } else {
        line.to_string()
    }
}

pub fn join_list_item_continuations(s: &str) -> String {
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    let mut i = 0usize;
    while i + 1 < lines.len() {
        let cur_t = lines[i].trim_start();
        let cur_is_list = cur_t.starts_with("- ")
            || cur_t.starts_with("* ")
            || cur_t.starts_with("\u{2022} ")
            || NUMBERED_LIST_ITEM_PREFIX.is_match(cur_t);
        if !cur_is_list {
            i += 1;
            continue;
        }
        let next_t = lines[i + 1].trim_start();
        if next_t.is_empty() {
            i += 1;
            continue;
        }
        if STRUCT_LINE_PREFIX.is_match(next_t) {
            i += 1;
            continue;
        }
        let first = next_t.chars().next().expect("non-empty");
        if !(first.is_lowercase() || first.is_alphanumeric()) {
            i += 1;
            continue;
        }
        if next_t.len() <= 3 && next_t.chars().all(|c| c.is_ascii_digit()) {
            i += 1;
            continue;
        }
        let cur_end = lines[i].trim_end().to_string();
        let last_char = cur_end.chars().last().unwrap_or(' ');
        if matches!(last_char, '.' | '!' | '?') {
            i += 1;
            continue;
        }
        lines[i] = format!("{cur_end} {next_t}");
        lines.remove(i + 1);
    }
    lines.join("\n")
}

#[allow(dead_code)]
pub fn join_pdf_wrapped_prose_lines(s: &str) -> String {
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    let mut i = 0usize;
    while i + 1 < lines.len() {
        if should_join_prose_pair(&lines[i], &lines[i + 1]) {
            let joiner = if soft_hyphen_join(&lines[i], &lines[i + 1]) {
                String::new()
            } else {
                String::from(" ")
            };
            let next = lines[i + 1].trim_start().to_string();
            lines[i] = format!("{}{joiner}{next}", lines[i].trim_end());
            lines.remove(i + 1);
            continue;
        }
        i += 1;
    }
    lines.join("\n")
}

#[allow(dead_code)]
fn should_join_prose_pair(cur: &str, next: &str) -> bool {
    let c = cur.trim_end();
    let n = next.trim_start();
    let both_present = !c.is_empty() && !n.is_empty();
    let no_struct = !STRUCT_LINE_PREFIX.is_match(c) && !STRUCT_LINE_PREFIX.is_match(n);
    let no_colon = !c.ends_with(':') && !c.ends_with(';');
    if !(both_present && no_struct && no_colon) {
        return false;
    }
    let words: Vec<&str> = c.split_whitespace().collect();
    let word_count = words.len();
    if word_count > 0 && word_count <= 6 && c.len() <= 50 {
        let title_case = words
            .iter()
            .filter(|w| w.chars().next().is_some_and(char::is_uppercase))
            .count();
        if title_case >= word_count.div_ceil(2) {
            return false;
        }
    }
    let last_char = c.chars().last().expect("non-empty");
    let first_char = n.chars().next().expect("non-empty");
    let last_ok = last_char.is_alphanumeric() || last_char == ',' || last_char == '-';
    let first_ok = first_char.is_alphanumeric() || first_char == '(' || first_char == '"';
    if !(last_ok && first_ok) {
        return false;
    }
    let n_first_word: String = n.chars().take_while(|ch| ch.is_alphabetic()).collect();
    let n_continues = matches!(
        n_first_word.to_lowercase().as_str(),
        "and" | "or" | "but" | "by" | "to" | "of" | "in" | "on" | "at" | "for" | "with"
            | "from" | "the" | "a" | "an" | "that" | "which" | "who" | "as"
    );
    c.len() >= 50 || last_char == '-' || last_char == ',' || first_char.is_lowercase() || n_continues
}

#[allow(dead_code)]
fn soft_hyphen_join(cur: &str, next: &str) -> bool {
    let c = cur.trim_end();
    let n = next.trim_start();
    let last_word = c.split_whitespace().next_back().unwrap_or("");
    if last_word.len() < 4 {
        return false;
    }
    let last_word_alpha = last_word.chars().all(char::is_alphabetic);
    if !last_word_alpha {
        return false;
    }
    let first_word: String = n
        .chars()
        .take_while(|ch| ch.is_alphabetic())
        .collect();
    if first_word.len() < 2 || first_word.len() > 5 {
        return false;
    }
    let last_lower = last_word.to_lowercase();
    has_suffix3(&last_lower, "fac")
        || has_suffix3(&last_lower, "tio")
        || has_suffix3(&last_lower, "atu")
        || has_suffix3(&last_lower, "ati")
        || has_suffix3(&last_lower, "tra")
        || has_suffix3(&last_lower, "rva")
        || has_suffix3(&last_lower, "stu")
        || has_suffix3(&last_lower, "uli")
        || has_suffix3(&last_lower, "rta")
        || has_suffix3(&last_lower, "esu")
        || has_suffix3(&last_lower, "ani")
}

#[allow(dead_code)]
fn has_suffix3(s: &str, suffix: &str) -> bool {
    s.chars().rev().take(3).collect::<String>().chars().rev().collect::<String>() == suffix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bullet_marker_below_text_moves_up() {
        let s = "alpha text\n\u{2022}\nbeta text\n\u{2022}\n";
        let o = move_trailing_bullet_markers_onto_previous_line(s);
        assert!(o.contains("\u{2022} alpha text"), "{o}");
        assert!(o.contains("\u{2022} beta text"), "{o}");
    }

    #[test]
    fn numbered_marker_below_text_moves_up() {
        let s = "Do thing one\n1.\nDo thing two\n2.\n";
        let o = move_trailing_numbered_markers_onto_previous_line(s);
        assert!(o.contains("1. Do thing one"), "{o}");
        assert!(o.contains("2. Do thing two"), "{o}");
    }

    #[test]
    fn promotes_known_section_headings() {
        let s = "Executive Summary\nbody\nFindings\nmore";
        let o = promote_plain_section_headings(s);
        assert!(o.contains("# Executive Summary"));
        assert!(o.contains("## Findings"));
    }

    #[test]
    fn joins_pdf_wrapped_prose_lines_with_space() {
        let s = "this report presents organizational performance for\nfiscal year";
        let o = join_pdf_wrapped_prose_lines(s);
        assert_eq!(o, "this report presents organizational performance for fiscal year");
    }

    #[test]
    fn does_not_join_into_heading_or_list() {
        let s = "intro paragraph\n# Heading";
        assert_eq!(join_pdf_wrapped_prose_lines(s), s);
        let s2 = "lead text\n- item";
        assert_eq!(join_pdf_wrapped_prose_lines(s2), s2);
    }
}
