use std::path::PathBuf;
use std::process::Command;

#[test]
fn parse_out_of_range_page_yields_exit_3() {
    let exe = env!("CARGO_BIN_EXE_rpdf");
    let pdf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    let out = std::env::temp_dir().join(format!("rpdf_partial_{}.md", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let status = Command::new(exe)
        .arg("parse")
        .arg("--pages")
        .arg("1,999")
        .arg("--output")
        .arg(&out)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert_eq!(status.code(), Some(3));
    assert!(std::fs::metadata(&out).is_ok());
    let _ = std::fs::remove_file(&out);
}
