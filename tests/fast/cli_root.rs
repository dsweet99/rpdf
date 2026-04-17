use std::process::Command;

use super::common;

#[test]
fn version_flag_prints_rpdf() {
    let out = Command::new(common::exe())
        .arg("--version")
        .output()
        .expect("run rpdf");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(stdout.starts_with("rpdf "));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    assert!(stdout.contains(&format!("pdfium={}", rpdf::PDFIUM_BINARY_TAG)));
}

#[test]
fn subprocess_exercises_binary_main_entrypoint() {
    let out = Command::new(common::exe())
        .output()
        .expect("run rpdf with no args");
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn root_help_exits_zero() {
    let out = Command::new(common::exe())
        .arg("--help")
        .output()
        .expect("run");
    assert!(out.status.success());
}

#[test]
fn parse_help_exits_zero() {
    let out = Command::new(common::exe())
        .args(["parse", "--help"])
        .output()
        .expect("run");
    assert!(out.status.success());
}

#[test]
fn inspect_help_exits_zero() {
    let out = Command::new(common::exe())
        .args(["inspect", "--help"])
        .output()
        .expect("run");
    assert!(out.status.success());
}

#[test]
fn unknown_subcommand_exits_one() {
    let out = Command::new(common::exe())
        .arg("not-a-real-subcommand")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
}
