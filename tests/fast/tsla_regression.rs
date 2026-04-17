use std::fs;
use std::process::Command;

use super::common;

fn forbidden_control_chars(s: &str) -> Vec<u32> {
    let mut out: Vec<u32> = s
        .chars()
        .filter(|&ch| ch != '\n' && ch != '\r' && ch != '\t')
        .filter(|&ch| ch.is_control())
        .map(u32::from)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn json_text_content(raw: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(raw).expect("json");
    let mut joined = String::new();
    if let Some(pages) = v.get("pages").and_then(serde_json::Value::as_array) {
        for page in pages {
            if let Some(elements) = page.get("elements").and_then(serde_json::Value::as_array) {
                for el in elements {
                    if let Some(text) = el.get("text").and_then(serde_json::Value::as_str) {
                        if !joined.is_empty() {
                            joined.push('\n');
                        }
                        joined.push_str(text);
                    }
                }
            }
        }
    }
    joined
}

#[test]
fn tsla_markdown_has_no_embedded_control_characters() {
    let pdf = common::tsla_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_tsla_reg_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let out = dir.join("tsla.md");
    let result = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&out)
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );

    let md = fs::read_to_string(&out).expect("read markdown");
    let bad = forbidden_control_chars(&md);
    assert!(
        bad.is_empty(),
        "unexpected embedded control chars in tsla markdown: {bad:?}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tsla_json_text_has_no_embedded_control_characters() {
    let pdf = common::tsla_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_tsla_json_reg_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let out_md = dir.join("tsla.md");
    let out_json = dir.join("tsla.json");
    let result = Command::new(common::exe())
        .arg("parse")
        .arg("--output")
        .arg(&out_md)
        .arg("--json")
        .arg(&out_json)
        .arg(&pdf)
        .output()
        .expect("spawn");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );

    let raw = fs::read_to_string(&out_json).expect("read json");
    let text = json_text_content(&raw);
    let bad = forbidden_control_chars(&text);
    assert!(
        bad.is_empty(),
        "unexpected embedded control chars in tsla json text: {bad:?}"
    );

    let _ = fs::remove_dir_all(&dir);
}
