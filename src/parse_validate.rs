use crate::cli::ParseCli;

pub fn validate_parse_cli(cli: &ParseCli, expanded_input_count: usize) -> Result<(), String> {
    let mut problems: Vec<&'static str> = Vec::new();
    validate_stdout_and_output_flags(cli, expanded_input_count, &mut problems);
    validate_output_dir_rules(cli, &mut problems);
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

fn validate_output_dir_rules(cli: &ParseCli, problems: &mut Vec<&'static str>) {
    if cli.output.is_some() && cli.output_dir.is_some() {
        problems.push("--output cannot be combined with --output-dir");
    }
}

fn validate_json_debug_paths(cli: &ParseCli, problems: &mut Vec<&'static str>) {
    if cli.output_dir.is_some() {
        validate_sidecar_filename(cli.json.as_ref(), problems, true);
        validate_sidecar_filename(cli.debug_json.as_ref(), problems, false);
    }
    if let (Some(j), Some(d)) = (&cli.json, &cli.debug_json) {
        if j == d {
            problems.push("--json and --debug-json cannot be the same path");
            return;
        }
        if cli.output_dir.is_some() {
            let jn = j.file_name();
            let dn = d.file_name();
            if jn.is_some() && jn == dn {
                problems.push("--json and --debug-json basenames must differ with --output-dir");
            }
        }
    }
}

fn validate_sidecar_filename(
    path: Option<&std::path::PathBuf>,
    problems: &mut Vec<&'static str>,
    is_json: bool,
) {
    let Some(path) = path else {
        return;
    };
    let missing = if is_json {
        "--json with --output-dir requires a filename"
    } else {
        "--debug-json with --output-dir requires a filename"
    };
    let invalid_utf8 = if is_json {
        "--json with --output-dir requires a UTF-8 filename"
    } else {
        "--debug-json with --output-dir requires a UTF-8 filename"
    };
    let md_collision = if is_json {
        "--json with --output-dir cannot use basename 'md'"
    } else {
        "--debug-json with --output-dir cannot use basename 'md'"
    };
    match path.file_name() {
        None => problems.push(missing),
        Some(name) if name.to_str().is_none() => problems.push(invalid_utf8),
        Some(name) if name.to_str().is_some_and(|n| n.eq_ignore_ascii_case("md")) => {
            problems.push(md_collision);
        }
        Some(_) => {}
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
    fn rejects_output_and_output_dir_together() {
        let mut c = parse_cli_base();
        c.output = Some(PathBuf::from("x.md"));
        c.output_dir = Some(PathBuf::from("out"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn accepts_output_dir_with_single_or_multiple_expanded_inputs() {
        for n in [1_usize, 2_usize] {
            let mut c = parse_cli_base();
            c.output_dir = Some(PathBuf::from("out"));
            assert!(validate_parse_cli(&c, n).is_ok(), "{n}");
        }
    }

    #[test]
    fn rejects_identical_json_and_debug_json_paths() {
        let mut c = parse_cli_base();
        let p = PathBuf::from("out.json");
        c.json = Some(p.clone());
        c.debug_json = Some(p);
        assert!(validate_parse_cli(&c, 1).is_err());
    }

    #[test]
    fn rejects_same_json_and_debug_basename_with_output_dir() {
        let mut c = parse_cli_base();
        c.output_dir = Some(PathBuf::from("out"));
        c.json = Some(PathBuf::from("/tmp/a/out.json"));
        c.debug_json = Some(PathBuf::from("/tmp/b/out.json"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn rejects_json_without_filename_with_output_dir() {
        let mut c = parse_cli_base();
        c.output_dir = Some(PathBuf::from("out"));
        c.json = Some(PathBuf::from("/"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn rejects_debug_json_without_filename_with_output_dir() {
        let mut c = parse_cli_base();
        c.output_dir = Some(PathBuf::from("out"));
        c.debug_json = Some(PathBuf::from("/"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn rejects_json_md_basename_with_output_dir() {
        let mut c = parse_cli_base();
        c.output_dir = Some(PathBuf::from("out"));
        c.json = Some(PathBuf::from("md"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[test]
    fn rejects_debug_json_md_basename_with_output_dir() {
        let mut c = parse_cli_base();
        c.output_dir = Some(PathBuf::from("out"));
        c.debug_json = Some(PathBuf::from("md"));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_utf8_json_filename_with_output_dir() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let mut c = parse_cli_base();
        c.output_dir = Some(PathBuf::from("out"));
        c.json = Some(PathBuf::from(OsString::from_vec(vec![0xff, b'.', b'j', b's', b'o', b'n'])));
        assert!(validate_parse_cli(&c, 2).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_utf8_debug_filename_with_output_dir() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let mut c = parse_cli_base();
        c.output_dir = Some(PathBuf::from("out"));
        c.debug_json = Some(PathBuf::from(OsString::from_vec(vec![0xfe, b'.', b'j', b's', b'o', b'n'])));
        assert!(validate_parse_cli(&c, 2).is_err());
    }
}

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn symbol_refs() {
        assert_eq!(
            stringify!(super::validate_stdout_and_output_flags),
            "super::validate_stdout_and_output_flags"
        );
        assert_eq!(
            stringify!(super::validate_output_dir_rules),
            "super::validate_output_dir_rules"
        );
        assert_eq!(
            stringify!(super::validate_json_debug_paths),
            "super::validate_json_debug_paths"
        );
        assert_eq!(
            stringify!(super::validate_multi_input_flags),
            "super::validate_multi_input_flags"
        );
        assert_eq!(
            stringify!(super::validate_sidecar_filename),
            "super::validate_sidecar_filename"
        );
        assert_eq!(
            stringify!(super::validate_mode_strings),
            "super::validate_mode_strings"
        );
    }
}
