use std::process::Command;

use super::common;

#[test]
fn render_command_is_stub() {
    let out = Command::new(common::exe())
        .args(["render", "x.pdf", "--page", "1", "--output", "y.png"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).expect("utf8");
    assert!(stderr.contains("not implemented"));
}
