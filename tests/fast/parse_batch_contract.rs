use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::common;

fn batch_output_dir_two_pdfs(temp_prefix: &str, block_one_md: bool) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("{temp_prefix}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("one.pdf");
    let b = dir.join("two.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).expect("mkdir");
    if block_one_md {
        fs::write(out.join("one.md"), b"x\n").expect("seed md");
    }
    let dummy = dir.join("dummy.json");
    (dir, out, a, b, dummy)
}

#[test]
fn parse_batch_two_inputs_writes_output_dir() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_batch_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("one.pdf");
    let b = dir.join("two.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out");
    let _ = fs::remove_dir_all(&out);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(0));
    let md1 = out.join("one.md");
    let md2 = out.join("two.md");
    assert!(md1.is_file());
    assert!(md2.is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_batch_output_dir_writes_per_stem_json() {
    let (dir, out, a, b, dummy) = batch_output_dir_two_pdfs("rpdf_json_batch", false);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg("--json")
        .arg(&dummy)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(0));
    assert!(out.join("one.json").is_file());
    assert!(out.join("two.json").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_batch_does_not_write_json_when_stem_markdown_is_blocked() {
    let (dir, out, a, b, dummy) = batch_output_dir_two_pdfs("rpdf_batch_json_md_order", true);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg("--json")
        .arg(&dummy)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(3));
    assert!(!out.join("one.json").exists());
    assert!(out.join("two.json").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn batch_parse_refuses_to_overwrite_existing_per_stem_json() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!(
        "rpdf_batch_noclobber_json_sidecar_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("one.pdf");
    let b = dir.join("two.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).expect("mkdir");
    fs::write(out.join("one.json"), b"{}\n").expect("seed json");
    let dummy = dir.join("dummy.json");
    let output = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg("--json")
        .arg(&dummy)
        .arg(&a)
        .arg(&b)
        .output()
        .expect("spawn");
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(fs::read_to_string(out.join("one.json")).expect("read json"), "{}\n");
    assert!(out.join("two.json").is_file());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("refusing to overwrite"),
        "expected overwrite detail on stderr, got: {stderr:?}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_batch_output_dir_writes_per_stem_debug_json() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_dbg_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("one.pdf");
    let b = dir.join("two.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out");
    let _ = fs::remove_dir_all(&out);
    let dummy = dir.join("dummy.json");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg("--debug-json")
        .arg(&dummy)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(0));
    assert!(out.join("one.debug.json").is_file());
    assert!(out.join("two.debug.json").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_batch_without_output_dir_writes_next_to_inputs() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_implicit_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("alpha.pdf");
    let b = dir.join("beta.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let md_a = dir.join("alpha.md");
    let md_b = dir.join("beta.md");
    let _ = fs::remove_file(&md_a);
    let _ = fs::remove_file(&md_b);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(0));
    assert!(md_a.is_file());
    assert!(md_b.is_file());
    let _ = fs::remove_file(&md_a);
    let _ = fs::remove_file(&md_b);
    let _ = fs::remove_dir_all(&dir);
}
