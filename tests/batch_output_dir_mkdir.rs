use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn batch_parse_fails_fast_when_output_dir_cannot_be_created() {
    let exe = env!("CARGO_BIN_EXE_rpdf");
    let pdf_a = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    let pdf_b = std::env::temp_dir().join(format!("rpdf_batch_dup_{}.pdf", std::process::id()));
    fs::copy(&pdf_a, &pdf_b).expect("copy fixture for second distinct input");
    let blocker = std::env::temp_dir().join(format!(
        "rpdf_batch_dir_blocker_{}",
        std::process::id()
    ));
    let _ = fs::remove_file(&blocker);
    fs::write(&blocker, b"x").expect("create file blocking output-dir path");
    let status = Command::new(exe)
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
