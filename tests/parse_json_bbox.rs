use std::path::PathBuf;
use std::process::Command;

#[test]
fn parse_json_element_bbox_is_text_union_not_full_page() {
    let exe = env!("CARGO_BIN_EXE_rpdf");
    let pdf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    let out = std::env::temp_dir().join(format!("rpdf_bbox_{}.json", std::process::id()));
    let md = std::env::temp_dir().join(format!("rpdf_bbox_{}.md", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&md);
    let status = Command::new(exe)
        .arg("parse")
        .arg("--json")
        .arg(&out)
        .arg("--output")
        .arg(&md)
        .arg(&pdf)
        .status()
        .expect("spawn");
    assert!(status.success());
    let raw = std::fs::read_to_string(&out).expect("read json");
    let doc_json: serde_json::Value = serde_json::from_str(&raw).expect("json");
    let bbox = doc_json["pages"][0]["elements"][0]["bbox"]
        .as_array()
        .expect("bbox");
    assert_eq!(bbox.len(), 4);
    let page_w = doc_json["pages"][0]["width"].as_f64().expect("width");
    let page_h = doc_json["pages"][0]["height"].as_f64().expect("height");
    let area_page = page_w * page_h;
    let left = bbox[0].as_f64().expect("left");
    let bottom = bbox[1].as_f64().expect("bottom");
    let right = bbox[2].as_f64().expect("right");
    let top = bbox[3].as_f64().expect("top");
    let area_box = (right - left).max(0.0) * (top - bottom).max(0.0);
    assert!(
        area_box < area_page * 0.99,
        "expected text union bbox smaller than full page, got area_box={area_box} area_page={area_page}"
    );
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&md);
}
