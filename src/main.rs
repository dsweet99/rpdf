#![allow(clippy::multiple_crate_versions)]

fn main() {
    std::process::exit(rpdf::run_from_args(std::env::args_os()));
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    #[test]
    fn main_wires_run_from_args_snapshot() {
        let code = rpdf::run_from_args(
            [OsString::from("rpdf"), OsString::from("--version")]
                .into_iter(),
        );
        assert_eq!(code, 0);
    }

    #[test]
    fn main_entry_symbol_exists() {
        let _: fn() = super::main;
    }
}
