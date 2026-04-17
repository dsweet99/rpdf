use crate::cli::ParseCli;

#[must_use]
pub fn parse_cli_base() -> ParseCli {
    ParseCli {
        inputs: vec![],
        output: None,
        json: None,
        stdout: false,
        output_dir: None,
        pages: None,
        password: None,
        use_struct_tree: false,
        reading_order: None,
        table_mode: None,
        include_header_footer: false,
        keep_line_breaks: false,
        quiet: false,
        debug_json: None,
    }
}
