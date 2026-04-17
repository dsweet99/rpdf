use std::fs;
use std::process::Command;

use super::common;

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
    assert!(raw.contains("\"status\""));
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
    assert!(raw.contains("\"status\""));
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
    assert!(raw.contains("\"status\""));
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
