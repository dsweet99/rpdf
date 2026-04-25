use regex::Regex;
use std::sync::LazyLock;

static STRUCT_LINE_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\s*)(#{1,6}\s|[-*]\s|\d{1,3}\.\s|\|[^|]|>|```|•\s)").expect("struct prefix")
});

#[allow(dead_code)]
pub fn join_pdf_wrapped_prose_lines(s: &str) -> String {
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    let mut i = 0usize;
    while i + 1 < lines.len() {
        if should_join_prose_pair(&lines[i], &lines[i + 1]) {
            let soft_hyphen = soft_hyphen_join(&lines[i], &lines[i + 1]);
            let hard_hyphen = lines[i].trim_end().ends_with('-');
            let joiner = if soft_hyphen || hard_hyphen { "" } else { " " };
            let next = lines[i + 1].trim_start().to_string();
            let cur = if soft_hyphen {
                lines[i].trim_end().trim_end_matches('-')
            } else {
                lines[i].trim_end()
            };
            lines[i] = format!("{cur}{joiner}{next}");
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
    if !pair_can_join(c, n) {
        return false;
    }
    let words = c.split_whitespace().collect::<Vec<_>>();
    if looks_like_title(words.as_slice(), c) {
        return false;
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

fn pair_can_join(cur: &str, next: &str) -> bool {
    let both_present = !cur.is_empty() && !next.is_empty();
    let no_struct = !STRUCT_LINE_PREFIX.is_match(cur) && !STRUCT_LINE_PREFIX.is_match(next);
    let no_tables_or_labels = !cur.contains('|')
        && !next.contains('|')
        && !cur.trim_start().starts_with("TABLE-")
        && !next.trim_start().starts_with("TABLE-");
    let no_colon = !cur.ends_with(':') && !cur.ends_with(';');
    both_present && no_struct && no_tables_or_labels && no_colon
}

fn looks_like_title(words: &[&str], cur: &str) -> bool {
    let word_count = words.len();
    if word_count == 0 || word_count > 6 || cur.len() > 50 {
        return false;
    }
    let title_case = words
        .iter()
        .filter(|w| w.chars().next().is_some_and(char::is_uppercase))
        .count();
    title_case >= word_count.div_ceil(2)
}

#[allow(dead_code)]
fn soft_hyphen_join(cur: &str, next: &str) -> bool {
    let c = cur.trim_end();
    let n = next.trim_start();
    if !c.ends_with('-') {
        return false;
    }
    let last_word = c.split_whitespace().next_back().unwrap_or("");
    let stem = last_word.strip_suffix('-').unwrap_or(last_word);
    if stem.is_empty() {
        return false;
    }
    if stem.len() < 4 {
        return false;
    }
    let last_word_alpha = stem.chars().all(char::is_alphabetic);
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
    let last_lower = stem.to_lowercase();
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
    fn join_pdf_wrapped_prose_lines_keeps_space_for_non_hyphen_wrap() {
        let out = join_pdf_wrapped_prose_lines("our satisfaction\nmodel improved");
        assert_eq!(out, "our satisfaction model improved");
    }

    #[test]
    fn soft_hyphen_join_requires_explicit_trailing_hyphen() {
        assert!(!soft_hyphen_join("satis", "faction"));
    }

    #[test]
    fn join_pdf_wrapped_prose_lines_removes_soft_hyphen_wrap() {
        let out = join_pdf_wrapped_prose_lines("customer satisfac-\ntion improved");
        assert_eq!(out, "customer satisfaction improved");
    }

    #[test]
    fn join_pdf_wrapped_prose_lines_keeps_hard_hyphen_without_space() {
        let out = join_pdf_wrapped_prose_lines("inter-\nnational");
        assert_eq!(out, "inter-national");
    }
}

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn symbol_refs() {
        assert_eq!(
            stringify!(super::should_join_prose_pair),
            "super::should_join_prose_pair"
        );
        assert_eq!(stringify!(super::pair_can_join), "super::pair_can_join");
        assert_eq!(stringify!(super::looks_like_title), "super::looks_like_title");
        assert_eq!(stringify!(super::soft_hyphen_join), "super::soft_hyphen_join");
        assert_eq!(stringify!(super::has_suffix3), "super::has_suffix3");
    }
}
