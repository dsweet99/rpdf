use std::process::Command;

use super::common;

#[test]
fn inspect_reports_pages_on_stdout() {
    let pdf = common::sample_pdf();
    let out = Command::new(common::exe())
        .arg("inspect")
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(stdout.contains("pages:"));
    assert!(stdout.contains("mark_info_dictionary_probe:"));
    assert!(stdout.contains("structure_tree_root_probe:"));
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(!stderr.contains("warning:"));
}

#[test]
fn inspect_with_pages_filter_succeeds() {
    let pdf = common::sample_pdf();
    let out = Command::new(common::exe())
        .arg("inspect")
        .arg("--pages")
        .arg("1")
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(stdout.contains("pages:"));
}

#[test]
fn inspect_rejects_invalid_pages_spec() {
    let pdf = common::sample_pdf();
    let out = Command::new(common::exe())
        .arg("inspect")
        .arg("--pages")
        .arg("not-a-page")
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(1));
}
