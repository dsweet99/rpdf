pub fn reflow_pipe_table_text(s: &str) -> String {
    let mut s = s.replace("| |", "|\n|");
    s = s.replace(". |", ".\n|");
    s = s.replace("| # ", "|\n# ");
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

fn merge_adjacent_separator_rows(lines: &mut Vec<String>) {
    let mut i = 0usize;
    while i + 1 < lines.len() {
        if separator_row_is_dash_cells(&lines[i]) && separator_row_is_dash_cells(&lines[i + 1]) {
            let merged = merge_two_dash_rows(&lines[i], &lines[i + 1]);
            lines[i] = merged;
            lines.remove(i + 1);
            continue;
        }
        i += 1;
    }
}

fn merge_two_dash_rows(a: &str, b: &str) -> String {
    let count_a = a
        .split('|')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .count();
    let count_b = b
        .split('|')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .count();
    let total = count_a + count_b;
    let inner = std::iter::repeat_n("-", total)
        .collect::<Vec<_>>()
        .join(" | ");
    format!("| {inner} |")
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
    if !first.contains('|') && !lines[1].contains('|') {
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
}
