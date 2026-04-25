use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(unix)]
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

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
    assert!(out.join("one.dummy.json").is_file());
    assert!(out.join("two.dummy.json").is_file());
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
    assert!(!out.join("one.dummy.json").exists());
    assert!(out.join("two.dummy.json").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn batch_parse_refuses_to_overwrite_existing_per_stem_json() {
    run_sidecar_noclobber_case("rpdf_batch_noclobber_json_sidecar", "--json");
}

#[test]
fn batch_parse_refuses_to_overwrite_existing_per_stem_debug_json_without_markdown_side_effect() {
    run_sidecar_noclobber_case("rpdf_batch_noclobber_debug_sidecar", "--debug-json");
}

fn run_sidecar_noclobber_case(prefix: &str, sidecar_flag: &str) {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("{prefix}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("one.pdf");
    let b = dir.join("two.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).expect("mkdir");
    fs::write(out.join("one.dummy.json"), b"{}\n").expect("seed debug json");
    let dummy = dir.join("dummy.json");
    let output = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg(sidecar_flag)
        .arg(&dummy)
        .arg(&a)
        .arg(&b)
        .output()
        .expect("spawn");
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(
        fs::read_to_string(out.join("one.dummy.json")).expect("read debug"),
        "{}\n"
    );
    assert!(!out.join("one.md").exists());
    assert!(out.join("two.dummy.json").is_file());
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
    assert!(out.join("one.dummy.json").is_file());
    assert!(out.join("two.dummy.json").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_batch_output_dir_disambiguates_duplicate_stems() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_dup_stem_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let a_dir = dir.join("a");
    let b_dir = dir.join("b");
    fs::create_dir_all(&a_dir).expect("mkdir a");
    fs::create_dir_all(&b_dir).expect("mkdir b");
    let a = a_dir.join("report.pdf");
    let b = b_dir.join("report.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg(&a)
        .arg(&b)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(0));
    assert!(out.join("report.md").is_file());
    assert!(out.join("report-1.md").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_batch_output_dir_avoids_suffix_stem_collisions() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_dup_suffix_stem_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let a_dir = dir.join("a");
    let b_dir = dir.join("b");
    let c_dir = dir.join("c");
    fs::create_dir_all(&a_dir).expect("mkdir a");
    fs::create_dir_all(&b_dir).expect("mkdir b");
    fs::create_dir_all(&c_dir).expect("mkdir c");
    let a = a_dir.join("report.pdf");
    let b = b_dir.join("report.pdf");
    let c = c_dir.join("report-1.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    fs::copy(&sample, &c).expect("copy c");
    let out = dir.join("out");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg(&a)
        .arg(&b)
        .arg(&c)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(0));
    assert!(out.join("report.md").is_file());
    assert!(out.join("report-1.md").is_file());
    assert!(out.join("report-2.md").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn parse_batch_output_dir_handles_non_utf8_input_filename() {
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_non_utf8_stem_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let mut bytes = b"weird-".to_vec();
    bytes.push(0xFF);
    bytes.extend_from_slice(b".pdf");
    let input = dir.join(OsString::from_vec(bytes));
    fs::copy(&sample, &input).expect("copy");
    let out = dir.join("out");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output-dir")
        .arg(&out)
        .arg(&input)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(0));
    assert!(out.join("input-1.md").is_file());
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
