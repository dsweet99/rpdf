use super::common;

#[test]
#[ignore = "slow: RPDF_RUN_SLOW=1 cargo test --test slow -- --ignored"]
fn neighboring_benchmark_repo_probe() {
    common::require_slow_gate();
    let repo = common::benchmark_repo();
    if !repo.is_dir() {
        return;
    }
    let out = common::benchmark_python(
        r#"import os
from pdf_bench.loader import get_parser_info, load_parsers
from pdf_bench.config import BenchmarkConfig, DocumentFilter, MetricConfig, OutputConfig, ParserConfig

info = get_parser_info("rpdf")
assert info["id"] == "rpdf"

cfg = BenchmarkConfig(
    name="rpdf neighbor probe",
    description="rpdf neighbor probe",
    corpus_dir="corpus",
    document_filter=DocumentFilter(),
    parser_config=ParserConfig(
        parsers=["rpdf"],
        parser_options={"rpdf": {"binary": os.environ["RPDF_BIN"]}},
    ),
    metric_config=MetricConfig(metrics=["edit_distance"]),
    output_config=OutputConfig(),
)
parsers = load_parsers(cfg)
assert len(parsers) == 1
assert parsers[0].name == "rpdf"
assert parsers[0].version
print(parsers[0].version)
"#,
    )
        .output()
        .expect("spawn python");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(!stdout.trim().is_empty());
}
