use std::fs;
use std::process::Command;

use super::common;

#[test]
fn parse_reading_order_off_writes_markdown() {
    let pdf = common::sample_pdf();
    let out = std::env::temp_dir().join(format!(
        "rpdf_parse_roff_{}.md",
        std::process::id()
    ));
    let _ = fs::remove_file(&out);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--reading-order")
        .arg("off")
        .arg("--output")
        .arg(&out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(status.success());
    let md = fs::read_to_string(&out).expect("read md");
    assert!(!md.trim().is_empty());
    let _ = fs::remove_file(&out);
}

#[test]
fn parse_writes_markdown_for_sample_pdf() {
    let pdf = common::sample_pdf();
    let out = std::env::temp_dir().join(format!("rpdf_parse_fixture_{}.md", std::process::id()));
    let _ = fs::remove_file(&out);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(status.success());
    let md = fs::read_to_string(&out).expect("read md");
    assert!(!md.trim().is_empty());
    let _ = fs::remove_file(&out);
}

#[test]
fn parse_without_flags_writes_markdown_next_to_input() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_default_md_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let pdf = dir.join("hello.pdf");
    fs::copy(&sample, &pdf).expect("copy");
    let md = dir.join("hello.md");
    let _ = fs::remove_file(&md);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(status.success());
    assert!(md.is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_refuses_to_overwrite_existing_markdown() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_noclobber_md_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let out = dir.join("out.md");
    fs::write(&out, b"blocker\n").expect("seed");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&out)
        .arg(&sample)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    assert_eq!(fs::read_to_string(&out).expect("read"), "blocker\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_does_not_write_json_when_default_markdown_path_is_blocked() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_noclobber_json_after_md_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let pdf = dir.join("doc.pdf");
    let md = dir.join("doc.md");
    let json_sidecar = dir.join("side.json");
    fs::copy(&sample, &pdf).expect("copy");
    fs::write(&md, b"blocker\n").expect("seed md");
    let _ = fs::remove_file(&json_sidecar);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--json")
        .arg(&json_sidecar)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(1));
    assert!(!json_sidecar.exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_refuses_to_overwrite_existing_json_sidecar() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_noclobber_json_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let md = dir.join("out.md");
    let json_out = dir.join("out.json");
    fs::write(&json_out, b"{}\n").expect("seed json");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&md)
        .arg("--json")
        .arg(&json_out)
        .arg(&sample)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(2));
    assert_eq!(fs::read_to_string(&json_out).expect("read json"), "{}\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_refuses_to_overwrite_existing_debug_json_sidecar() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_noclobber_dbg_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let md = dir.join("out.md");
    let dbg = dir.join("out.debug.json");
    fs::write(&dbg, b"{}\n").expect("seed dbg");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&md)
        .arg("--debug-json")
        .arg(&dbg)
        .arg(&sample)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(2));
    assert_eq!(fs::read_to_string(&dbg).expect("read dbg"), "{}\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn batch_parse_refuses_to_overwrite_existing_markdown() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_batch_noclobber_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("one.pdf");
    let b = dir.join("two.pdf");
    fs::copy(&sample, &a).expect("copy");
    fs::copy(&sample, &b).expect("copy");
    let out = dir.join("out");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).expect("mkdir");
    fs::write(out.join("one.md"), b"x\n").expect("seed md");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(3));
    assert_eq!(fs::read_to_string(out.join("one.md")).expect("read"), "x\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn batch_parse_fails_fast_when_output_dir_cannot_be_created() {
    let pdf_a = common::sample_pdf();
    let pdf_b = std::env::temp_dir().join(format!("rpdf_batch_dup_{}.pdf", std::process::id()));
    fs::copy(&pdf_a, &pdf_b).expect("copy fixture for second distinct input");
    let blocker = std::env::temp_dir().join(format!(
        "rpdf_batch_dir_blocker_{}",
        std::process::id()
    ));
    let _ = fs::remove_file(&blocker);
    fs::write(&blocker, b"x").expect("create file blocking output-dir path");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg(&pdf_a)
        .arg(&pdf_b)
        .arg("--output-dir")
        .arg(&blocker)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(2));
    let _ = fs::remove_file(&blocker);
    let _ = fs::remove_file(&pdf_b);
}

#[test]
fn batch_creates_output_dir_when_absent() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_batch_mkdir_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("solo.pdf");
    let b = dir.join("twin.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("fresh_out");
    let _ = fs::remove_dir_all(&out);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert!(status.success());
    assert!(out.join("solo.md").is_file());
    assert!(out.join("twin.md").is_file());
    let _ = fs::remove_dir_all(&dir);
}
