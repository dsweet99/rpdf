use std::path::PathBuf;
use std::process::Command;

#[test]
fn parse_writes_markdown_for_sample_pdf() {
    let exe = env!("CARGO_BIN_EXE_rpdf");
    let pdf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample.pdf");
    let out = std::env::temp_dir().join(format!("rpdf_parse_fixture_{}.md", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let status = Command::new(exe)
        .arg("parse")
        .arg("--output")
        .arg(&out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(status.success());
    let md = std::fs::read_to_string(&out).expect("read md");
    assert!(!md.trim().is_empty());
    let _ = std::fs::remove_file(&out);
}
