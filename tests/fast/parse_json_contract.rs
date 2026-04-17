use std::fs;
use std::process::Command;

use super::common;

fn stable_json_fields(raw: &str) -> serde_json::Value {
    let mut v: serde_json::Value = serde_json::from_str(raw).expect("json");
    if let Some(obj) = v.as_object_mut() {
        obj.remove("input");
    }
    v
}

#[test]
fn parse_json_element_bbox_is_four_finite_numbers() {
    let pdf = common::sample_pdf();
    let out = std::env::temp_dir().join(format!("rpdf_bbox_{}.json", std::process::id()));
    let md = std::env::temp_dir().join(format!("rpdf_bbox_{}.md", std::process::id()));
    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&md);
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--json")
        .arg(&out)
        .arg("--output")
        .arg(&md)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(status.success());
    let raw = fs::read_to_string(&out).expect("read json");
    let doc_json: serde_json::Value = serde_json::from_str(&raw).expect("json");
    let bbox = doc_json["pages"][0]["elements"][0]["bbox"]
        .as_array()
        .expect("bbox");
    assert_eq!(bbox.len(), 4);
    for x in bbox {
        let v = x.as_f64().expect("bbox coord");
        assert!(v.is_finite());
    }
    assert_eq!(
        doc_json["pages"][0]["elements"][0]["type"].as_str(),
        Some("paragraph")
    );
    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&md);
}

#[test]
fn parse_json_has_required_top_level_fields() {
    let pdf = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_json_shape_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let json_out = dir.join("out.json");
    let md = dir.join("out.md");
    let status = Command::new(common::exe())
        .arg("parse")
        .arg("--json")
        .arg(&json_out)
        .arg("--output")
        .arg(&md)
        .arg("--reading-order")
        .arg("xycut")
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(status.success());
    let raw = fs::read_to_string(&json_out).expect("read json");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    for key in [
        "schema_version",
        "parser_version",
        "pdfium_binary_tag",
        "status",
        "input",
        "page_count",
        "warnings",
        "failed_pages",
        "config",
        "pages",
    ] {
        assert!(
            v.get(key).is_some(),
            "missing top-level key {key}"
        );
    }
    let page0 = &v["pages"][0];
    for key in ["page", "width", "height", "elements"] {
        assert!(page0.get(key).is_some(), "missing page key {key}");
    }
    let el0 = &page0["elements"][0];
    for key in ["id", "type", "page", "bbox", "text"] {
        assert!(el0.get(key).is_some(), "missing element key {key}");
    }
    let cfg = v["config"].as_object().expect("config object");
    assert_eq!(cfg["reading_order"], "xycut");
    assert!(!v["warnings"].as_array().expect("warnings").is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_stdout_is_stable_across_runs() {
    let pdf = common::sample_pdf();
    let mut a = Vec::new();
    let mut b = Vec::new();
    for buf in [&mut a, &mut b] {
        let out = Command::new(common::exe())
            .arg("parse")
            .arg("--stdout")
            .arg(&pdf)
            .output()
            .expect("spawn");
        assert!(out.status.success());
        buf.extend_from_slice(&out.stdout);
    }
    assert_eq!(a, b);
}

#[test]
fn parse_json_stable_fields_match_across_runs() {
    let pdf = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_json_det_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let j1 = dir.join("a.json");
    let j2 = dir.join("b.json");
    let md1 = dir.join("out1.md");
    let md2 = dir.join("out2.md");
    let _ = fs::remove_file(&md1);
    let _ = fs::remove_file(&md2);
    let s1 = Command::new(common::exe())
        .arg("parse")
        .arg("--json")
        .arg(&j1)
        .arg("--output")
        .arg(&md1)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(s1.success());
    let s2 = Command::new(common::exe())
        .arg("parse")
        .arg("--json")
        .arg(&j2)
        .arg("--output")
        .arg(&md2)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(s2.success());
    let s1 = fs::read_to_string(&j1).expect("read");
    let s2 = fs::read_to_string(&j2).expect("read");
    assert_eq!(stable_json_fields(&s1), stable_json_fields(&s2));
    let _ = fs::remove_dir_all(&dir);
}
