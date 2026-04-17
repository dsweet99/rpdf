use std::fs;

use super::common;

#[test]
#[ignore = "slow: RPDF_RUN_SLOW=1 cargo test --test slow -- --ignored"]
fn benchmark_side_adapter_writes_markdown_at_output_path() {
    common::require_slow_gate();
    let repo = common::benchmark_repo();
    if !repo.is_dir() {
        return;
    }
    let pdf = common::sample_pdf();
    let dir = std::env::temp_dir().join(format!("rpdf_bench_smoke_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    let input = dir.join(format!("doc_{}.pdf", std::process::id()));
    fs::copy(&pdf, &input).expect("copy");
    let out_dir = dir.join("bench_out");
    fs::create_dir_all(&out_dir).expect("mkdir out");
    let output = common::benchmark_python(
        r#"import os
from pathlib import Path
from pdf_bench.config import BenchmarkConfig, DocumentFilter, MetricConfig, OutputConfig, ParserConfig
from pdf_bench.loader import load_parsers

cfg = BenchmarkConfig(
    name="rpdf smoke",
    description="rpdf smoke",
    corpus_dir="corpus",
    document_filter=DocumentFilter(),
    parser_config=ParserConfig(
        parsers=["rpdf"],
        parser_options={"rpdf": {"binary": os.environ["RPDF_BIN"]}},
    ),
    metric_config=MetricConfig(metrics=["edit_distance"]),
    output_config=OutputConfig(),
)
parser = load_parsers(cfg)[0]
pdf_path = Path(os.environ["RPDF_SAMPLE_PDF"])
output_dir = Path(os.environ["RPDF_OUTPUT_DIR"])
result = parser.parse(pdf_path, output_dir)
assert result == output_dir / f"{pdf_path.stem}.md"
assert result.exists()
assert result.read_text(encoding="utf-8").strip()
print(result)
"#,
    )
    .env("RPDF_SAMPLE_PDF", &input)
    .env("RPDF_OUTPUT_DIR", &out_dir)
    .output()
    .expect("spawn python");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out_md = out_dir.join(format!("{}.md", input.file_stem().unwrap().to_string_lossy()));
    assert!(out_md.is_file());
    let _ = fs::remove_dir_all(&dir);
}
