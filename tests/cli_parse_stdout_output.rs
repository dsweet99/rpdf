use std::path::PathBuf;
use std::process::Command;

#[test]
fn parse_rejects_stdout_with_output() {
    let exe = env!("CARGO_BIN_EXE_rpdf");
    let pdf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    let out = std::env::temp_dir().join(format!("rpdf_stdout_conflict_{}.md", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let status = Command::new(exe)
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
