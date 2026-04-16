#![allow(clippy::multiple_crate_versions)]

mod cli;
mod engine;
mod expand;
mod inspect_cmd;
mod markdown;
mod model;
mod pagespec;
mod parse_cmd;
mod parse_document;

pub use engine::PDFIUM_BINARY_TAG;
pub use model::normalize_text;

#[must_use]
pub const fn version_string() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[must_use]
pub fn run_from_args<I>(it: I) -> i32
where
    I: Iterator<Item = std::ffi::OsString>,
{
    use clap::Parser;
    let argv: Vec<std::ffi::OsString> = it.collect();
    match cli::Root::try_parse_from(&argv) {
        Ok(root) => match root.command {
            cli::Commands::Parse(p) => parse_cmd::run_parse(&p),
            cli::Commands::Inspect(i) => inspect_cmd::run_inspect(&i),
        },
        Err(e) => {
            use clap::error::ErrorKind;
            match e.kind() {
                ErrorKind::DisplayVersion => {
                    println!(
                        "rpdf {} pdfium={}",
                        env!("CARGO_PKG_VERSION"),
                        engine::PDFIUM_BINARY_TAG
                    );
                    0
                }
                ErrorKind::DisplayHelp => {
                    print!("{e}");
                    0
                }
                _ => {
                    eprintln!("{e}");
                    1
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn version_is_non_empty_semver_like() {
        let v = version_string();
        assert!(!v.is_empty());
        assert!(v.chars().all(|c| c.is_ascii_digit() || c == '.'));
    }

    #[test]
    fn version_flag_returns_0() {
        let args = vec![OsString::from("rpdf"), OsString::from("--version")];
        assert_eq!(run_from_args(args.into_iter()), 0);
    }

    #[test]
    fn short_version_flag_returns_0() {
        let args = vec![OsString::from("rpdf"), OsString::from("-V")];
        assert_eq!(run_from_args(args.into_iter()), 0);
    }

    #[test]
    fn no_subcommand_shows_help() {
        let args = vec![OsString::from("rpdf")];
        assert_eq!(run_from_args(args.into_iter()), 1);
    }

    #[test]
    fn root_help_flag_returns_0() {
        let args = vec![OsString::from("rpdf"), OsString::from("--help")];
        assert_eq!(run_from_args(args.into_iter()), 0);
    }

    #[test]
    fn parse_help_flag_returns_0() {
        let args = vec![
            OsString::from("rpdf"),
            OsString::from("parse"),
            OsString::from("--help"),
        ];
        assert_eq!(run_from_args(args.into_iter()), 0);
    }
}
