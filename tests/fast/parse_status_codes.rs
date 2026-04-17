use std::fs;
use std::process::Command;

use super::common;

#[test]
fn parse_out_of_range_page_yields_exit_3() {
    let pdf = common::sample_pdf();
    let out = std::env::temp_dir().join(format!("rpdf_partial_{}.md", std::process::id()));
    let _ = fs::remove_file(&out);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--pages")
        .arg("1,999")
        .arg("--output")
        .arg(&out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(3));
    assert!(fs::metadata(&out).is_ok());
    let _ = fs::remove_file(&out);
}

#[test]
fn parse_invalid_pdf_bytes_yields_exit_2() {
    let dir = std::env::temp_dir().join(format!("rpdf_bad_pdf_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let bad = dir.join("bad.pdf");
    fs::write(&bad, b"not a pdf").expect("write");
    let out = dir.join("out.md");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&out)
        .arg(&bad)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(2));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_only_out_of_range_pages_yields_exit_2() {
    let pdf = common::sample_pdf();
    let out = std::env::temp_dir().join(format!("rpdf_all_bad_pages_{}.md", std::process::id()));
    let _ = fs::remove_file(&out);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--pages")
        .arg("999")
        .arg("--output")
        .arg(&out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(2));
    let _ = fs::remove_file(&out);
}
