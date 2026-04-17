use std::ffi::OsStr;
use std::process::Command;
use std::path::PathBuf;

pub const fn rpdf_exe() -> &'static str {
    env!("CARGO_BIN_EXE_rpdf")
}

pub fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn sample_pdf() -> PathBuf {
    manifest_dir().join("tests/fixtures/sample.pdf")
}

pub fn benchmark_repo() -> PathBuf {
    manifest_dir().join("../pdf-parser-benchmark")
}

pub fn benchmark_python(script: &str) -> Command {
    let repo = benchmark_repo();
    let mut pythonpath = repo.clone().into_os_string();
    if let Some(existing) = std::env::var_os("PYTHONPATH") {
        pythonpath.push(OsStr::new(":"));
        pythonpath.push(existing);
    }

    let mut cmd = Command::new("python");
    cmd.current_dir(&repo)
        .arg("-c")
        .arg(script)
        .env("PYTHONPATH", pythonpath)
        .env("RPDF_BIN", rpdf_exe());
    cmd
}

pub fn require_slow_gate() {
    assert_eq!(
        std::env::var_os("RPDF_RUN_SLOW").as_deref(),
        Some(OsStr::new("1")),
        "set RPDF_RUN_SLOW=1 when running slow tests with --ignored"
    );
}
