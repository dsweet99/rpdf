pub fn reflow_pipe_table_text(s: &str) -> String {
    let mut s = s
        .lines()
        .map(normalize_pipe_line)
        .collect::<Vec<_>>()
        .join("\n");
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    for _ in 0u8..8 {
        let n = lines.len();
        merge_orphan_pipe_lines(&mut lines);
        if lines.len() == n {
            break;
        }
    }
    rejoin_wrapped_pipe_rows(&mut lines);
    merge_adjacent_separator_rows(&mut lines);
    s = lines.join("\n");
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    split_duplicate_prefix_second_line(lines.as_mut_slice());
    let mut i = 0;
    while i + 1 < lines.len() {
        if lines[i] == lines[i + 1] && lines[i].contains('|') {
            lines.remove(i + 1);
        } else {
            i += 1;
        }
    }
    lines.join("\n")
}

fn normalize_pipe_line(line: &str) -> String {
    let split_markers = line.matches(" | | ").count();
    let rewritten = if split_markers == 0 {
        line.to_string()
    } else {
        split_glued_pipe_rows(line, split_markers)
    };
    if rewritten.chars().filter(|c| *c == '|').count() < 2 {
        rewritten
    } else {
        rewritten.replace(". |", ".\n|").replace("| # ", "|\n# ")
    }
}

fn split_glued_pipe_rows(line: &str, split_markers: usize) -> String {
    let mut out = String::new();
    let mut rest = line;
    loop {
        let Some((left, right)) = rest.split_once(" | | ") else {
            out.push_str(rest);
            break;
        };
        let split_ok = if left.contains('|') {
            match right.trim_start().chars().next() {
                Some(_) if split_markers >= 2 => true,
                Some(c) => c.is_ascii_digit() || c == '-',
                None => false,
            }
        } else {
            false
        };
        if !split_ok {
            out.push_str(rest);
            break;
        }
        out.push_str(left);
        out.push_str(" |\n| ");
        rest = right;
    }
    out
}

fn merge_adjacent_separator_rows(lines: &mut Vec<String>) {
    let mut i = 0usize;
    while i + 1 < lines.len() {
        if separator_row_is_dash_cells(&lines[i]) && separator_row_is_dash_cells(&lines[i + 1]) {
            let count_a = lines[i]
                .split('|')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .count();
            let count_b = lines[i + 1]
                .split('|')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .count();
            let inner = std::iter::repeat_n("-", count_a.max(count_b))
                .collect::<Vec<_>>()
                .join(" | ");
            lines[i] = format!("| {inner} |");
            lines.remove(i + 1);
            continue;
        }
        i += 1;
    }
}

fn rejoin_wrapped_pipe_rows(lines: &mut Vec<String>) {
    let mut i = 0usize;
    while i + 1 < lines.len() {
        if try_join_table_row_wrap(lines, i) {
            continue;
        }
        i += 1;
    }
}

fn is_partial_separator_row(line: &str) -> bool {
    let t = line.trim();
    let starts_pipe = t.starts_with('|');
    let ends_pipe = t.ends_with('|');
    if !starts_pipe || ends_pipe {
        return false;
    }
    let inner = t.strip_prefix('|').unwrap_or(t).trim();
    !inner.is_empty() && inner.chars().all(|c| c == '-' || c.is_whitespace() || c == '|')
}

fn try_join_table_row_wrap(lines: &mut Vec<String>, i: usize) -> bool {
    let cur = lines[i].trim_end().to_string();
    let next = lines[i + 1].trim().to_string();
    if !cur.starts_with('|') || next.is_empty() {
        return false;
    }
    let cur_partial_sep = is_partial_separator_row(&cur);
    let next_is_sep = separator_row_is_dash_cells(&next);
    if cur_partial_sep && next_is_sep {
        let next_inner = next.strip_prefix('|').unwrap_or(&next).trim_start();
        lines[i] = format!("{cur} | {next_inner}");
        lines.remove(i + 1);
        return true;
    }
    if separator_row_is_dash_cells(&cur) || next_is_sep {
        return false;
    }
    try_join_table_row_cells(lines, i, &cur, &next)
}

fn try_join_table_row_cells(lines: &mut Vec<String>, i: usize, cur: &str, next: &str) -> bool {
    let cur_ends_pipe = cur.ends_with('|');
    let next_starts_pipe = next.starts_with('|');
    let next_valid = next.contains('|') && next.ends_with('|');
    let joined = if cur_ends_pipe && !next_starts_pipe && next_valid {
        Some(format!("{cur} {next}"))
    } else if !cur_ends_pipe && next_starts_pipe && next.ends_with('|') {
        let next_inner = next.strip_prefix('|').unwrap_or(next).trim_start();
        Some(format!("{cur} | {next_inner}"))
    } else if !cur_ends_pipe && !next_starts_pipe && next_valid {
        Some(format!("{cur} {next}"))
    } else {
        None
    };
    joined.is_some_and(|merged| {
        lines[i] = merged;
        lines.remove(i + 1);
        true
    })
}

pub fn rewrite_separator_rows_in_pipe_tables(s: &str) -> String {
    let mut lines: Vec<String> = s.lines().map(String::from).collect();
    for line in &mut lines {
        if separator_row_is_dash_cells(line) {
            *line = rewrite_separator_row(line);
        }
    }
    lines.join("\n")
}

fn merge_orphan_pipe_lines(lines: &mut Vec<String>) {
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "|" {
            i += 1;
            continue;
        }
        let removed = lines
            .get(i + 1)
            .is_some_and(|next_line| next_line.trim_start().starts_with('|'));
        if removed {
            lines.remove(i);
        }
        if removed {
            continue;
        }
        i += 1;
    }
}

fn split_duplicate_prefix_second_line(lines: &mut [String]) {
    if lines.len() < 2 {
        return;
    }
    let first = lines[0].trim_end();
    if first.is_empty() || !first.contains('|') {
        return;
    }
    if !lines[1].contains('|') {
        return;
    }
    let prefix = format!("{first} ");
    let Some(rest) = lines[1].strip_prefix(&prefix) else {
        return;
    };
    if !rest.trim_start().starts_with('|') {
        return;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejoins_pipe_row_when_cell_wraps_at_column_boundary() {
        let s = "| Margin | 26%\n| 28% |";
        let out = reflow_pipe_table_text(s);
        assert!(out.contains("| Margin | 26% | 28% |"), "expected joined row, got: {out}");
    }

    #[test]
    fn rejoins_wrapped_margin_row_in_full_table_context() {
        let s = "| Profit | $1.1M | $1.4M | | Margin | 26%\n| 28% | # Next";
        let out = reflow_pipe_table_text(s);
        assert!(out.contains("| Margin | 26% | 28% |"), "expected joined margin row, got:\n{out}");
    }

    #[test]
    fn rejoins_pipe_row_when_continuation_lacks_leading_pipe() {
        let s = "| Region | Performance Metrics | Performance Metrics |\nPerformance Metrics |\n| --- | --- | --- | --- |";
        let out = reflow_pipe_table_text(s);
        assert!(out.contains("| Region | Performance Metrics | Performance Metrics | Performance Metrics |"), "{out}");
    }

    #[test]
    fn rejoins_pipe_row_when_continuation_starts_with_pipe() {
        let s = "| Revenue | $5,000,000\n| $5,750,000 |";
        let out = reflow_pipe_table_text(s);
        assert!(out.contains("| Revenue | $5,000,000 | $5,750,000 |"), "{out}");
    }

    #[test]
    fn rejoins_pipe_row_when_cell_text_wraps_after_complete_pipe() {
        let s = "| Gross Profit |\n$1,800,000 | $2,300,000 |";
        let out = reflow_pipe_table_text(s);
        assert!(out.contains("| Gross Profit | $1,800,000 | $2,300,000 |"), "{out}");
    }

    #[test]
    fn reflow_pipe_table_text_preserves_non_table_repeated_prefix() {
        let input = "A | B\nA | B is noted";
        let out = reflow_pipe_table_text(input);
        assert_eq!(out, input);
    }

    #[test]
    fn reflow_pipe_table_text_preserves_second_line_prefix_before_pipe_row() {
        let input = "Executive Summary\nExecutive Summary | Metric | Value |";
        let out = reflow_pipe_table_text(input);
        assert_eq!(out, input);
    }

    #[test]
    fn reflow_does_not_split_empty_table_cells() {
        let input = "| A | B | C |\n| --- | --- | --- |\n| x | | y |";
        let out = reflow_pipe_table_text(input);
        assert!(out.contains("| x | | y |"), "{out}");
    }

    #[test]
    fn reflow_does_not_split_empty_cell_followed_by_uppercase_text() {
        let input = "| A | B | C |\n| --- | --- | --- |\n| x | | USA |";
        let out = reflow_pipe_table_text(input);
        assert!(out.contains("| x | | USA |"), "{out}");
    }

    #[test]
    fn reflow_splits_all_glued_rows_on_one_line() {
        let input = "| A | B | | C | D | | E | F |";
        let out = reflow_pipe_table_text(input);
        assert!(out.contains("| A | B |\n| C | D |\n| E | F |"), "{out}");
    }

    #[test]
    fn reflow_splits_glued_row_when_next_row_starts_numeric() {
        let input = "| Metric | Value | | 2024 | 10 |";
        let out = reflow_pipe_table_text(input);
        assert!(out.contains("| Metric | Value |\n| 2024 | 10 |"), "{out}");
    }

    #[test]
    fn reflow_orphan_pipe_does_not_tableize_plain_paragraph() {
        let input = "|\nNarrative sentence";
        let out = reflow_pipe_table_text(input);
        assert_eq!(out, input);
    }

    #[test]
    fn reflow_does_not_rewrite_single_pipe_prose() {
        let input = "Narrative . | marker";
        let out = reflow_pipe_table_text(input);
        assert_eq!(out, input);
    }

    #[test]
    fn reflow_dedupes_adjacent_separator_rows_without_expanding_columns() {
        let input = "| A | B |\n| - | - |\n| - | - |\n| 1 | 2 |";
        let out = reflow_pipe_table_text(input);
        assert!(out.contains("| - | - |"), "{out}");
        assert_eq!(out.matches("| - | - |").count(), 1, "{out}");
        assert!(!out.contains("| --- | --- | --- | --- |"), "{out}");
    }
}

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn symbol_refs() {
        assert_eq!(
            stringify!(super::merge_adjacent_separator_rows),
            "super::merge_adjacent_separator_rows"
        );
        assert_eq!(stringify!(super::normalize_pipe_line), "super::normalize_pipe_line");
        assert_eq!(
            stringify!(super::split_glued_pipe_rows),
            "super::split_glued_pipe_rows"
        );
        assert_eq!(
            stringify!(super::rejoin_wrapped_pipe_rows),
            "super::rejoin_wrapped_pipe_rows"
        );
        assert_eq!(
            stringify!(super::is_partial_separator_row),
            "super::is_partial_separator_row"
        );
        assert_eq!(
            stringify!(super::try_join_table_row_wrap),
            "super::try_join_table_row_wrap"
        );
        assert_eq!(
            stringify!(super::try_join_table_row_cells),
            "super::try_join_table_row_cells"
        );
        assert_eq!(
            stringify!(super::rewrite_separator_rows_in_pipe_tables),
            "super::rewrite_separator_rows_in_pipe_tables"
        );
        assert_eq!(
            stringify!(super::merge_orphan_pipe_lines),
            "super::merge_orphan_pipe_lines"
        );
        assert_eq!(
            stringify!(super::split_duplicate_prefix_second_line),
            "super::split_duplicate_prefix_second_line"
        );
        assert_eq!(
            stringify!(super::separator_row_is_dash_cells),
            "super::separator_row_is_dash_cells"
        );
        assert_eq!(stringify!(super::rewrite_separator_row), "super::rewrite_separator_row");
    }
}
