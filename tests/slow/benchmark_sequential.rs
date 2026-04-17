use std::fs;

use super::common;

#[test]
#[ignore = "slow: RPDF_RUN_SLOW=1 cargo test --test slow -- --ignored"]
fn sequential_batch_parses_share_output_directory() {
    common::require_slow_gate();
    let repo = common::benchmark_repo();
    if !repo.is_dir() {
        return;
    }
    let sample = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_bench_seq_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    let c = dir.join("c.pdf");
    let d = dir.join("d.pdf");
    fs::copy(&sample, &a).expect("copy a");
    fs::copy(&sample, &b).expect("copy b");
    fs::copy(&sample, &c).expect("copy c");
    fs::copy(&sample, &d).expect("copy d");
    let out = dir.join("shared");
    fs::create_dir_all(&out).expect("mkdir out");
    let output = common::benchmark_python(
        r#"import os
from pathlib import Path
from pdf_bench.systems.rpdf_parser import RpdfParser

parser = RpdfParser(binary=os.environ["RPDF_BIN"])
output_dir = Path(os.environ["RPDF_OUTPUT_DIR"])
for key in ["RPDF_A", "RPDF_B", "RPDF_C", "RPDF_D"]:
    result = parser.parse(Path(os.environ[key]), output_dir)
    assert result.exists()
"#,
    )
    .env("RPDF_OUTPUT_DIR", &out)
    .env("RPDF_A", &a)
    .env("RPDF_B", &b)
    .env("RPDF_C", &c)
    .env("RPDF_D", &d)
    .output()
    .expect("spawn python");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out.join("a.md").is_file());
    assert!(out.join("b.md").is_file());
    assert!(out.join("c.md").is_file());
    assert!(out.join("d.md").is_file());
    let _ = fs::remove_dir_all(&dir);
}
