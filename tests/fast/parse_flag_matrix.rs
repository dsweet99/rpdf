use std::fs;
use std::process::Command;

use super::common;

fn setup_two_pdf_inputs(prefix: &str) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("{prefix}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    (dir, a, b)
}

#[test]
fn parse_rejects_stdout_with_output_dir() {
    let pdf = common::sample_pdf();
    let out = std::env::temp_dir().join(format!("rpdf_odir_{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--stdout")
        .arg("--output-dir")
        .arg(&out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
}

#[test]
fn parse_rejects_stdout_with_output() {
    let pdf = common::sample_pdf();
    let out = std::env::temp_dir().join(format!("rpdf_stdout_conflict_{}.md", std::process::id()));
    let _ = fs::remove_file(&out);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--stdout")
        .arg("--output")
        .arg(&out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    assert!(!out.exists());
}

#[test]
fn parse_rejects_output_flag_with_multiple_inputs_even_with_output_dir() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_out_multi_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out");
    let md = dir.join("merged.md");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&md)
        .arg("--output-dir")
        .arg(&out)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_rejects_json_when_single_directory_operand_expands_to_multiple_pdfs() {
    let base = std::env::temp_dir().join(format!("rpdf_json_dir_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("mkdir");
    fs::write(base.join("a.pdf"), b"%PDF-1.4\n%%EOF\n").expect("a");
    fs::write(base.join("b.pdf"), b"%PDF-1.4\n%%EOF\n").expect("b");
    let json_out = base.join("out.json");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--json")
        .arg(&json_out)
        .arg(&base)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn parse_accepts_json_when_directory_operand_expands_to_one_pdf() {
    let sample = common::sample_pdf();
    let base = std::env::temp_dir().join(format!("rpdf_json_dir_ok_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("mkdir");
    fs::copy(&sample, base.join("one.pdf")).expect("copy");
    let json_out = base.join("out.json");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--json")
        .arg(&json_out)
        .arg(&base)
        .status()
        .expect("spawn");
    assert!(status.success());
    let raw = fs::read_to_string(&json_out).expect("read json");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    assert!(parsed.get("status").is_some());
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn parse_rejects_json_with_multiple_explicit_inputs_without_output_dir() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_json_multi_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let json_out = dir.join("out.json");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--json")
        .arg(&json_out)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_rejects_debug_json_with_multiple_explicit_inputs_without_output_dir() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_dbg_multi_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out.debug.json");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--debug-json")
        .arg(&out)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_rejects_identical_json_and_debug_json_paths() {
    let pdf = common::sample_pdf();
    let out = std::env::temp_dir().join(format!("rpdf_same_json_{}.json", std::process::id()));
    let _ = fs::remove_file(&out);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--json")
        .arg(&out)
        .arg("--debug-json")
        .arg(&out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    assert!(!out.exists());
}

#[test]
fn parse_rejects_same_json_debug_basename_with_output_dir() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_same_base_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out");
    let json_out = dir.join("x").join("out.json");
    let dbg_out = dir.join("y").join("out.json");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg("--json")
        .arg(&json_out)
        .arg("--debug-json")
        .arg(&dbg_out)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_rejects_json_without_filename_with_output_dir() {
    let (dir, a, b) = setup_two_pdf_inputs("rpdf_json_basename");
    let out = dir.join("out");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg("--json")
        .arg(std::path::Path::new("/"))
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_rejects_json_md_basename_with_output_dir() {
    let (dir, a, b) = setup_two_pdf_inputs("rpdf_json_md_name");
    let out = dir.join("out");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg("--json")
        .arg("md")
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_rejects_stdout_when_directory_expands_to_multiple_pdfs() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_stdout_multi_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    fs::copy(&sample, dir.join("a.pdf")).expect("copy a");
    fs::copy(&sample, dir.join("b.pdf")).expect("copy b");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--stdout")
        .arg(&dir)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_rejects_invalid_reading_order() {
    let pdf = common::sample_pdf();
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--reading-order")
        .arg("bogus")
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
}

#[test]
fn parse_rejects_invalid_table_mode() {
    let pdf = common::sample_pdf();
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--table-mode")
        .arg("bogus")
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
}

#[test]
fn parse_single_stdout_and_json_succeeds() {
    let pdf = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_stdout_json_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let json_out = dir.join("out.json");
    let out = Command::new(common::exe())
        .arg("parse")
        .arg("--stdout")
        .arg("--json")
        .arg(&json_out)
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert!(out.status.success());
    assert!(!out.stdout.is_empty());
    let raw = fs::read_to_string(&json_out).expect("read json");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    assert!(parsed.get("status").is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_single_output_and_json_succeeds() {
    let pdf = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_out_json_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let md = dir.join("out.md");
    let json_out = dir.join("out.json");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&md)
        .arg("--json")
        .arg(&json_out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(status.success());
    let raw = fs::read_to_string(&json_out).expect("read json");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    assert!(parsed.get("status").is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_single_directory_one_pdf_allows_output_flags() {
    let sample = common::sample_pdf();
    let base = std::env::temp_dir().join(format!("rpdf_dir_one_sig_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("mkdir");
    fs::copy(&sample, base.join("only.pdf")).expect("copy");
    let md = base.join("out.md");
    let json_out = base.join("out.json");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&md)
        .arg("--json")
        .arg(&json_out)
        .arg(&base)
        .status()
        .expect("spawn");
    assert!(status.success());
    let _ = fs::remove_dir_all(&base);
}
