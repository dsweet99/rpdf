use std::path::PathBuf;

#[must_use]
pub const fn exe() -> &'static str {
    env!("CARGO_BIN_EXE_rpdf")
}

#[must_use]
pub fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[must_use]
pub fn sample_pdf() -> PathBuf {
    manifest_dir().join("tests/fixtures/sample.pdf")
}

#[must_use]
pub fn tsla_pdf() -> PathBuf {
    manifest_dir().join("tests/data/tsla.pdf")
}
