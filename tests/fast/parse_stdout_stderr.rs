use std::fs;
use std::process::Command;

use super::common;

#[test]
fn parse_stdout_writes_document_to_stdout() {
    let pdf = common::sample_pdf();
    let out = Command::new(common::exe())
        .arg("parse")
        .arg("--stdout")
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(!stdout.trim().is_empty());
}

#[test]
fn parse_non_stdout_does_not_write_markdown_to_stderr() {
    let pdf = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_stderr_md_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let out_path = dir.join("out.md");
    let out = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&out_path)
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    let md = fs::read_to_string(&out_path).expect("read md");
    assert!(!stderr.contains(md.trim()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn partial_success_summary_on_stderr() {
    let pdf = common::sample_pdf();
    let out_path = std::env::temp_dir().join(format!("rpdf_partial_stderr_{}.md", std::process::id()));
    let _ = fs::remove_file(&out_path);
    let out = Command::new(common::exe())
        .arg("parse")
        .arg("--pages")
        .arg("1,999")
        .arg("--output")
        .arg(&out_path)
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(stderr.contains("partial_success"));
    let _ = fs::remove_file(&out_path);
}

#[test]
fn quiet_suppresses_partial_success_stderr() {
    let pdf = common::sample_pdf();
    let out_path = std::env::temp_dir().join(format!("rpdf_quiet_partial_{}.md", std::process::id()));
    let _ = fs::remove_file(&out_path);
    let out = Command::new(common::exe())
        .arg("parse")
        .arg("--quiet")
        .arg("--pages")
        .arg("1,999")
        .arg("--output")
        .arg(&out_path)
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(!stderr.contains("partial_success"));
    let _ = fs::remove_file(&out_path);
}

#[test]
fn quiet_suppresses_stub_stderr_but_keeps_json_warnings() {
    let pdf = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_quiet_warn_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let json_out = dir.join("out.json");
    let md = dir.join("out.md");
    let out = Command::new(common::exe())
        .arg("parse")
        .arg("--quiet")
        .arg("--reading-order")
        .arg("xycut")
        .arg("--json")
        .arg(&json_out)
        .arg("--output")
        .arg(&md)
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(!stderr.contains("reading-order"));
    let raw = fs::read_to_string(&json_out).expect("read json");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    let warns = v["warnings"].as_array().expect("warnings");
    assert!(!warns.is_empty());
    let _ = fs::remove_dir_all(&dir);
}
