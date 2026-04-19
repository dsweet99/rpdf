use regex::Regex;
use std::sync::LazyLock;

pub fn dedup_repeated_title_line(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() < 2 {
        return s.to_string();
    }
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    let mut deduped_first = false;
    while i < lines.len() {
        if !deduped_first && i + 1 < lines.len() && consider_dedup(lines[i], lines[i + 1]) {
            if let Some(replacement) = dedup_pair(lines[i], lines[i + 1]) {
                out.extend(replacement);
                i += 2;
                deduped_first = true;
                continue;
            }
        }
        out.push(lines[i].to_string());
        i += 1;
    }
    out.join("\n")
}

fn consider_dedup(a_raw: &str, _b_raw: &str) -> bool {
    let a = a_raw.trim();
    !a.is_empty()
        && !a.starts_with('#')
        && !a.starts_with('-')
        && !a.starts_with('|')
        && a.chars().count() <= 120
}

fn dedup_pair(a_raw: &str, b_raw: &str) -> Option<Vec<String>> {
    let a = a_raw.trim();
    let b = b_raw.trim();
    if a == b {
        return Some(vec![a_raw.to_string()]);
    }
    let rest = b.strip_prefix(a)?.trim_start();
    if rest.is_empty() || a.len() < 10 && !rest.starts_with('#') {
        return if rest.is_empty() {
            Some(vec![a_raw.to_string()])
        } else if rest.starts_with('#') {
            Some(vec![a_raw.to_string(), rest.to_string()])
        } else {
            None
        };
    }
    if rest.starts_with('#') {
        return Some(vec![a_raw.to_string(), rest.to_string()]);
    }
    let body = rest.strip_prefix(a).map(str::trim_start);
    match body {
        Some(b) if !b.is_empty() => Some(vec![a_raw.to_string(), a.to_string(), b.to_string()]),
        Some(_) => Some(vec![a_raw.to_string(), a.to_string()]),
        None => Some(vec![a_raw.to_string(), rest.to_string()]),
    }
}

pub fn dedup_repeated_title_within_first_line(s: &str) -> String {
    let mut iter = s.splitn(2, '\n');
    let Some(first) = iter.next() else {
        return s.to_string();
    };
    let rest = iter.next();
    let trimmed = first.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 120 {
        return s.to_string();
    }
    if let Some(half_end) = halfway_repeat(trimmed) {
        let kept = trimmed[..half_end].trim_end().to_string();
        return match rest {
            Some(r) => format!("{kept}\n{r}"),
            None => kept,
        };
    }
    s.to_string()
}

fn halfway_repeat(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    if n >= 6 && n.is_multiple_of(2) {
        let mid = n / 2;
        let left: String = chars[..mid].iter().collect();
        let right: String = chars[mid..].iter().collect();
        if left.trim() == right.trim() {
            let byte_pos = s
                .char_indices()
                .nth(mid)
                .map_or(s.len(), |(i, _)| i);
            return Some(byte_pos);
        }
    }
    let bytes = s.as_bytes();
    for split in 3..n {
        let i = s.char_indices().nth(split).map(|(i, _)| i)?;
        let left = s[..i].trim();
        let right = s[i..].trim();
        if !left.is_empty() && left == right {
            return Some(i);
        }
        if left.len() >= 10 && right.starts_with(left) {
            return Some(i);
        }
        if i > bytes.len() / 2 + 4 {
            break;
        }
    }
    None
}

pub fn blank_line_between_metadata_fields(s: &str) -> String {
    static FIELD: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(Author|Date|Report ID|Subject|From|To|Title|Version|Status|Reference)\b[^:]{0,20}:\s+\S")
            .expect("field")
    });
    let lines: Vec<&str> = s.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len() * 2);
    for (i, line) in lines.iter().enumerate() {
        if i > 0 && i < 12 {
            let cur_field = FIELD.is_match(line);
            let prev_field = FIELD.is_match(lines[i - 1]);
            let last_blank = out.last().is_some_and(|l| l.trim().is_empty());
            if cur_field && prev_field && !last_blank {
                out.push(String::new());
            }
        }
        out.push((*line).to_string());
    }
    out.join("\n")
}

#[allow(dead_code)]
pub fn has_atx_headings(s: &str) -> bool {
    s.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with("# ")
            || t.starts_with("## ")
            || t.starts_with("### ")
            || t.starts_with("#### ")
            || t.starts_with("##### ")
            || t.starts_with("###### ")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_repeated_title_with_trailing_content() {
        let s = "Quarterly Business Review Quarterly Business Review Q3 2025\nMore content";
        let out = dedup_repeated_title_within_first_line(s);
        assert!(out.starts_with("Quarterly Business Review\n"), "expected deduped title, got: {out}");
        assert!(!out.contains("Quarterly Business Review Quarterly"), "title still duplicated: {out}");
    }
}

#[cfg(test)]
mod syn_basic_001_tests {
    use super::*;
    use crate::model_postprocess::postprocess_extracted_markdown;

    #[test]
    fn dedup_preserves_two_title_instances() {
        let raw = "Introduction to PDF Parsing
Introduction to PDF Parsing Introduction to PDF Parsing PDF (Portable Document Format) is a file format";
        let s1 = dedup_repeated_title_line(raw);
        let s2 = dedup_repeated_title_within_first_line(&s1);
        assert_eq!(
            s2.matches("Introduction to PDF Parsing").count(),
            2,
            "Expected 2 title instances after dedup"
        );
    }

    #[test]
    fn full_postprocess_preserves_two_title_instances() {
        let raw = "Introduction to PDF Parsing
Introduction to PDF Parsing Introduction to PDF Parsing PDF (Portable Document Format) is a file format";
        let result = postprocess_extracted_markdown(raw);
        assert_eq!(
            result.matches("Introduction to PDF Parsing").count(),
            2,
            "Expected 2 title instances after full postprocess"
        );
    }
}
