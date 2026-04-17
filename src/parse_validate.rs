use crate::cli::ParseCli;

pub fn validate_parse_cli(cli: &ParseCli, expanded_input_count: usize) -> Result<(), String> {
    let mut problems: Vec<&'static str> = Vec::new();
    validate_stdout_and_output_flags(cli, expanded_input_count, &mut problems);
    validate_output_dir_rules(cli, expanded_input_count, &mut problems);
    validate_multi_input_flags(cli, expanded_input_count, &mut problems);
    validate_json_debug_paths(cli, &mut problems);
    validate_mode_strings(cli, &mut problems);
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

fn validate_stdout_and_output_flags(
    cli: &ParseCli,
    expanded_input_count: usize,
    problems: &mut Vec<&'static str>,
) {
    if cli.stdout && cli.output.is_some() {
        problems.push("--stdout cannot be combined with --output");
    }
    if cli.stdout && cli.output_dir.is_some() {
        problems.push("--stdout cannot be used with --output-dir");
    }
    if cli.stdout && expanded_input_count > 1 {
        problems.push("--stdout is only valid with a single expanded PDF input");
    }
}

fn validate_output_dir_rules(
    cli: &ParseCli,
    expanded_input_count: usize,
    problems: &mut Vec<&'static str>,
) {
    if cli.output.is_some() && cli.output_dir.is_some() {
        problems.push("--output cannot be combined with --output-dir");
    }
    if cli.output_dir.is_some() && expanded_input_count == 1 {
        problems.push("--output-dir requires multiple expanded PDF inputs");
    }
}

fn validate_json_debug_paths(cli: &ParseCli, problems: &mut Vec<&'static str>) {
    if let (Some(j), Some(d)) = (&cli.json, &cli.debug_json) {
        if j == d {
            problems.push("--json and --debug-json cannot be the same path");
        }
    }
}

fn validate_multi_input_flags(
    cli: &ParseCli,
    expanded_input_count: usize,
    problems: &mut Vec<&'static str>,
) {
    if cli.output.is_some() && expanded_input_count > 1 {
        problems.push("--output requires a single expanded PDF input");
    }
    if cli.json.is_some() && expanded_input_count > 1 && cli.output_dir.is_none() {
        problems.push("--json with multiple expanded PDF inputs requires --output-dir");
    }
    if cli.debug_json.is_some() && expanded_input_count > 1 && cli.output_dir.is_none() {
        problems.push("--debug-json with multiple expanded PDF inputs requires --output-dir");
    }
}

fn validate_mode_strings(cli: &ParseCli, problems: &mut Vec<&'static str>) {
    if cli.reading_order.as_deref().is_some_and(|s| {
        !matches!(s, "off" | "basic" | "xycut")
    }) {
        problems.push("invalid --reading-order (expected off|basic|xycut)");
    }
    if cli.table_mode.as_deref().is_some_and(|s| {
        !matches!(s, "off" | "lines" | "heuristic")
    }) {
        problems.push("invalid --table-mode (expected off|lines|heuristic)");
    }
}

#[cfg(test)]
mod tests {
    use super::validate_parse_cli;
    use crate::test_support::parse_cli_base;
    use std::path::PathBuf;

    #[test]
    fn rejects_stdout_with_output() {
        let mut c = parse_cli_base();
        c.stdout = true;
        c.output = Some(PathBuf::from("x.md"));
        assert!(validate_parse_cli(&c, 1).is_err());
    }

    #[test]
    fn rejects_stdout_with_output_dir() {
        let mut c = parse_cli_base();
        c.stdout = true;
        c.output_dir = Some(PathBuf::from("out"));
        assert!(validate_parse_cli(&c, 1).is_err());
    }

    #[test]
    fn rejects_stdout_with_multiple_expanded_inputs() {
        let mut c = parse_cli_base();
        c.stdout = true;
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn rejects_output_with_multiple_expanded_inputs() {
        let mut c = parse_cli_base();
        c.output = Some(PathBuf::from("x.md"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn rejects_json_with_multiple_inputs_without_output_dir() {
        let mut c = parse_cli_base();
        c.json = Some(PathBuf::from("x.json"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn rejects_debug_json_with_multiple_inputs_without_output_dir() {
        let mut c = parse_cli_base();
        c.debug_json = Some(PathBuf::from("x.debug.json"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn accepts_json_with_multiple_inputs_when_output_dir_set() {
        let mut c = parse_cli_base();
        c.json = Some(PathBuf::from("dummy.json"));
        c.output_dir = Some(PathBuf::from("out"));
        assert!(validate_parse_cli(&c, 2).is_ok());
    }

    #[test]
    fn rejects_invalid_reading_order() {
        let mut c = parse_cli_base();
        c.reading_order = Some("nope".to_string());
        assert!(validate_parse_cli(&c, 1).is_err());
    }

    #[test]
    fn rejects_invalid_table_mode() {
        let mut c = parse_cli_base();
        c.table_mode = Some("nope".to_string());
        assert!(validate_parse_cli(&c, 1).is_err());
    }

    #[test]
    fn accepts_valid_reading_order_values() {
        for v in ["off", "basic", "xycut"] {
            let mut c = parse_cli_base();
            c.reading_order = Some(v.to_string());
            assert!(validate_parse_cli(&c, 1).is_ok(), "{v}");
        }
    }

    #[test]
    fn accepts_valid_table_mode_values() {
        for v in ["off", "lines", "heuristic"] {
            let mut c = parse_cli_base();
            c.table_mode = Some(v.to_string());
            assert!(validate_parse_cli(&c, 1).is_ok(), "{v}");
        }
    }

    #[test]
    fn rejects_output_dir_with_single_expanded_input() {
        let mut c = parse_cli_base();
        c.output_dir = Some(PathBuf::from("out"));
        assert!(validate_parse_cli(&c, 1).is_err());
    }

    #[test]
    fn rejects_output_and_output_dir_together() {
        let mut c = parse_cli_base();
        c.output = Some(PathBuf::from("x.md"));
        c.output_dir = Some(PathBuf::from("out"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn accepts_output_dir_with_multiple_expanded_inputs() {
        let mut c = parse_cli_base();
        c.output_dir = Some(PathBuf::from("out"));
        assert!(validate_parse_cli(&c, 2).is_ok());
    }

    #[test]
    fn rejects_identical_json_and_debug_json_paths() {
        let mut c = parse_cli_base();
        let p = PathBuf::from("out.json");
        c.json = Some(p.clone());
        c.debug_json = Some(p);
        assert!(validate_parse_cli(&c, 1).is_err());
    }
}
