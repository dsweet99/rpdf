use std::fs;

use super::common;

#[test]
#[ignore = "slow: RPDF_RUN_SLOW=1 cargo test --test slow -- --ignored"]
fn bad_input_surfaces_as_failure() {
    common::require_slow_gate();
    let repo = common::benchmark_repo();
    if !repo.is_dir() {
        return;
    }
    let dir = std::env::temp_dir().join(format!("rpdf_bench_bad_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let bad = dir.join("bad.pdf");
    fs::write(&bad, b"nope").expect("write");
    let out = dir.join("out_dir");
    fs::create_dir_all(&out).expect("mkdir out");
    let output = common::benchmark_python(
        r#"import os
from pathlib import Path
from pdf_bench.systems.rpdf_parser import RpdfParser

parser = RpdfParser(binary=os.environ["RPDF_BIN"])
try:
    parser.parse(Path(os.environ["RPDF_BAD_PDF"]), Path(os.environ["RPDF_OUTPUT_DIR"]))
except RuntimeError as exc:
    print(exc)
else:
    raise SystemExit("expected RuntimeError from benchmark-side rpdf adapter")
"#,
    )
    .env("RPDF_BAD_PDF", &bad)
    .env("RPDF_OUTPUT_DIR", &out)
    .output()
    .expect("spawn python");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(&dir);
}
