use std::process::Command;

#[test]
fn version_flag_prints_rpdf() {
    let exe = env!("CARGO_BIN_EXE_rpdf");
    let out = Command::new(exe)
        .arg("--version")
        .output()
        .expect("run rpdf");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(stdout.starts_with("rpdf "));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    assert!(stdout.contains(&format!("pdfium={}", rpdf::PDFIUM_BINARY_TAG)));
}

#[test]
fn subprocess_exercises_binary_main_entrypoint() {
    let exe = env!("CARGO_BIN_EXE_rpdf");
    let out = Command::new(exe).output().expect("run rpdf with no args");
    assert_eq!(out.status.code(), Some(1));
}
