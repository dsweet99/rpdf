use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn parse_batch_two_inputs_writes_output_dir() {
    let exe = env!("CARGO_BIN_EXE_rpdf");
    let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    let dir = std::env::temp_dir().join(format!("rpdf_batch_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("one.pdf");
    let b = dir.join("two.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    let out = dir.join("out");
    let _ = fs::remove_dir_all(&out);
    let status = Command::new(exe)
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
