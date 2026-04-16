# rpdf eval test plan

## purpose

Turn the "product contract eval" work into durable automated tests so we can keep shipping parser changes without regressing the CLI contract, exit behavior, output layout, or basic JSON shape.

This plan is intentionally about eval track 1 from the earlier discussion:

- CLI and shell contract stability
- output artifact rules
- exit-code and partial-success behavior
- JSON contract and basic output-shape invariants
- inspect command smoke behavior

It is not yet a plan for full parser-quality scoring such as heading recall, reading-order quality, or table-structure accuracy. Those should come later once the parser is doing more than one-paragraph-per-page extraction.

## high-level strategy

Use two layers of tests:

- unit tests for pure logic, normalization, validation, and exit-code mapping
- integration tests for the real `rpdf` binary, filesystem behavior, stdout/stderr rules, and end-to-end artifacts

Use two speed tiers for integration tests:

- `tests/fast` for repo-local contract tests that should run routinely during local iteration and normal CI
- `tests/slow` for benchmark-facing compatibility tests that interact with `../pdf-parser-benchmark/` and may require more setup or runtime

The main rule is:

- if the behavior matters to a shell script, benchmark harness, or another tool invoking `rpdf`, prefer an integration test
- if the behavior is a pure function or helper with a small input/output surface, prefer a unit test

The benchmark-specific rule is:

- keep `rpdf`'s core contract suite self-contained and fast
- add a small number of slower compatibility tests that exercise the actual benchmark-side expectations from `../pdf-parser-benchmark/`
- do not turn the full benchmark corpus into part of the normal `cargo test` loop

## what we are trying to lock down

### 1. root CLI contract

We should preserve:

- `rpdf --version`
- `rpdf parse ...`
- `rpdf inspect ...`
- stable nonzero exits for invalid invocations

### 2. parse command contract

We should preserve:

- valid and invalid flag combinations
- single-input vs multi-input behavior
- directory expansion behavior
- `--stdout`, `--output`, `--json`, `--debug-json`, and `--output-dir` rules
- no silent overwrite behavior
- partial-success exit `3`

### 2a. benchmark compatibility contract

We should preserve the parts of `rpdf` that the benchmark side depends on:

- the benchmark-facing single-document flow maps cleanly to `rpdf parse <input.pdf> --output <output.md>`
- `rpdf --version` is callable and parseable enough for adapter metadata or logging
- successful parse writes a Markdown artifact at the caller-directed output path
- repeated sequential parses work against a shared output directory pattern
- parser failures surface as clear process failures rather than silent success with missing output

### 3. output artifact contract

We should preserve:

- default markdown path for single-input runs
- implicit output placement for batch runs without `--output-dir`
- per-stem file naming in batch mode
- JSON and debug JSON sidecar naming rules

### 4. JSON contract

We should preserve:

- required top-level fields
- stable status encoding
- required page and element structure
- normalized text and bbox shape invariants
- config echoing for parse-affecting options

### 5. inspect contract

We should preserve:

- inspect returns parseable human-readable metadata
- expected keys remain present
- stdout carries report data while warnings stay on stderr

## unit tests to add or expand

Unit tests should live close to the code they protect.

### `src/parse_validate.rs`

Add focused unit coverage for:

- `--stdout` with `--output`
- `--stdout` with `--output-dir`
- `--stdout` with multiple expanded inputs
- `--output` with multiple expanded inputs
- `--json` with multiple expanded inputs and no `--output-dir`
- `--debug-json` with multiple expanded inputs and no `--output-dir`
- invalid `--reading-order`
- invalid `--table-mode`
- valid `--reading-order` and `--table-mode` values

Reason:

- these are pure validation rules and should fail fast without spawning the binary

### `src/model.rs`

Add or keep unit coverage for:

- `RunStatus::exit_code()`
- `normalize_text()` CRLF and CR normalization
- serde representation of `RunStatus` as `snake_case`

Reason:

- these are small contract-critical helpers whose behavior should never drift silently

### `src/parse_cmd/mod.rs`

Add focused unit coverage for:

- `parse_config()` default values
- `parse_config()` propagation of explicit CLI values
- `load_pages_filter()` success and parse failure cases

Reason:

- these are pure or mostly pure helpers that define the normalized config recorded in JSON

### `src/parse_batch.rs`

Add unit coverage for:

- `status_outcome()`
- `batch_exit_code()` for all combinations:
- all success -> `0`
- some partial, none failed -> `3`
- some failed, some success -> `3`
- all failed -> `2`

Reason:

- batch exit behavior is part of the shell contract and easy to test directly

### `src/parse_document.rs`

Add limited unit coverage for pure helpers only:

- `write_atomic()` refuses overwrite
- `merge_filter_out_of_range_requests()` records warnings and failed pages
- stub warning helpers record the correct warnings for unsupported modes

Reason:

- these are contract-bearing helpers that do not need a full binary spawn

Avoid writing unit tests that depend on deep PDFium behavior unless the helper is already clearly isolated and deterministic.

## integration tests to add or expand

Integration tests should continue using the real compiled binary via `Command`.

## integration test tiers

### fast tests

Fast tests are the default contract-preservation layer:

- repo-local only
- use the compiled `rpdf` binary directly
- rely on checked-in fixtures or temp-dir fixtures
- expected to run in the normal local `cargo test` loop

### slow tests

Slow tests are benchmark-compatibility checks:

- interact with `../pdf-parser-benchmark/`
- may shell out to Python or the benchmark's test tooling
- may depend on benchmark fixtures or a local Python environment
- should be opt-in in day-to-day development, but runnable before benchmark-facing changes land

Design rule:

- slow tests should be a small smoke layer proving interoperability, not a duplicate of the full benchmark

Execution rule:

- slow tests must be explicitly gated so they do not run in the default local `cargo test` path by accident

### CLI contract tests

Keep or expand coverage for:

- `--version` prints `rpdf` version and PDFium tag
- root help returns `0`
- parse help returns `0`
- invalid subcommand usage returns `1`

These mostly already exist; they should remain simple smoke tests, but live under the fast test harness rather than staying as long-lived top-level integration tests.

### benchmark compatibility smoke tests

Add a small set of slow tests that directly interact with `../pdf-parser-benchmark/`.

Target behaviors:

- confirm the neighboring repo exists and expose a clean skip or explicit failure if the benchmark checkout is missing
- prove `rpdf` can satisfy the benchmark parser contract of "given `pdf_path` and `output_dir`, produce a Markdown file and return its path"
- prove repeated sequential parses through a benchmark-style output directory succeed
- prove benchmark-style callers can detect failure when `rpdf` fails on bad input

Two levels of benchmark-facing confidence:

- emulated benchmark compatibility: use a tiny Python snippet or helper that reproduces the benchmark's `parse(pdf_path, output_dir) -> Path` expectations while shelling out to `rpdf`
- real benchmark integration: exercise an actual `rpdf` adapter that lives on the benchmark side

For now:

- the slow suite may start with emulated benchmark compatibility checks
- the plan should eventually grow a real benchmark-side `rpdf` adapter and test that adapter directly

Preferred scope for the first version:

- use a tiny subset of benchmark-side expectations
- exercise one or a few synthetic PDFs rather than a broad corpus
- assert interoperability, not benchmark scores

Candidate slow tests:

- emulated benchmark contract smoke test on a tiny benchmark fixture if present, or on a tiny fallback fixture if the benchmark corpus is absent
- benchmark-style sequential parse of 2-3 documents into one output directory
- benchmark-style invalid-input failure propagation
- optional version smoke test that records `rpdf --version` output in a benchmark-style context
- later, real benchmark-adapter smoke test once the benchmark repo contains a thin `rpdf` adapter

Implementation approaches, in order of preference:

1. If we add a thin `rpdf` adapter on the benchmark side, call that adapter from the slow test and assert it returns a valid Markdown `Path`.
2. If the adapter does not exist yet, run a tiny Python snippet inside `../pdf-parser-benchmark/` that emulates the benchmark parser contract by shelling out to `rpdf parse ... --output ...`.
3. Only later, if useful, invoke a very small pytest target from the benchmark repo.

Guardrails:

- do not require the full benchmark corpus
- do not run the full benchmark from `cargo test`
- keep slow tests focused on contract compatibility and basic execution
- if benchmark fixtures are missing in the local checkout, skip cleanly or fall back to a tiny local fixture rather than hard-failing on a path assumption

### parse flag matrix tests

Keep current coverage and expand to include:

- `--json` plus multiple explicit inputs without `--output-dir` fails
- `--debug-json` plus multiple explicit inputs without `--output-dir` fails
- `--stdout` with a directory that expands to multiple PDFs fails
- invalid `--reading-order` fails with exit `1`
- invalid `--table-mode` fails with exit `1`
- single-input `--output <path> --json <path>` succeeds
- single-input `--stdout --json <path>` succeeds
- single-directory input that expands to one PDF can still be used with single-input output flags

These belong in integration tests because the real user-visible contract is the binary exit code plus stderr messaging.

### output path and overwrite tests

Add integration coverage for:

- single-input parse without flags writes next to input as `<stem>.md`
- single-input parse refuses to overwrite an existing markdown file
- single-input `--json` refuses to overwrite an existing JSON file
- single-input `--debug-json` refuses to overwrite an existing debug JSON file
- batch mode refuses to overwrite an existing target artifact
- `--output-dir` is created when absent
- `--output-dir` failure path returns the documented error exit

These are high-value regression tests because file clobbering or path drift will break scripting users quickly.

### stdout and stderr hygiene tests

Add integration coverage for:

- `--stdout` writes document markdown to stdout and not stderr
- diagnostics and warnings stay on stderr
- successful non-stdout parse does not print markdown to stderr
- partial-success run prints the partial-success summary only on stderr
- `--quiet` suppresses partial-success diagnostics on stderr
- `--quiet` suppresses stub warning chatter on stderr while preserving JSON `warnings`

This is especially important for benchmark wrappers and shell pipes.

### JSON contract tests

Add or expand integration coverage for:

- required top-level fields exist:
- `schema_version`
- `parser_version`
- `pdfium_binary_tag`
- `status`
- `input`
- `page_count`
- `warnings`
- `failed_pages`
- `config`
- `pages`
- required page fields exist:
- `page`
- `width`
- `height`
- `elements`
- required element fields exist for current paragraph output:
- `id`
- `type`
- `page`
- `bbox`
- `text`
- `bbox` is a 4-number array
- `config` reflects explicit CLI options in the emitted JSON
- unsupported options produce warnings in JSON rather than silently disappearing

Implementation note:

- prefer parsing the JSON with `serde_json::Value` and asserting only on stable contract fields, not the entire pretty-printed blob

### determinism tests

Add integration coverage for:

- running the same parse twice on the same fixture with `--stdout` yields identical stdout
- running the same parse twice with `--json` yields semantically identical JSON for stable fields

We should avoid brittle assertions on absolute temp paths. Compare fields that are intended to be stable, or normalize path-bearing fields before asserting equality.

### partial-success and failure tests

Keep or expand coverage for:

- out-of-range page request -> exit `3` with output present
- all requested pages invalid -> exit `2`
- full-failure tests should explicitly decide whether artifact absence is required, forbidden, or currently unspecified rather than assuming it from the exit code alone
- unreadable or invalid PDF input -> exit `2`
- invalid flag combination -> exit `1`

This is one of the highest-value parts of the contract because scripts need to distinguish usage error from parse failure from partial success.

### inspect tests

Keep or expand coverage for:

- inspect prints expected keys on stdout
- inspect with valid `--pages` works
- inspect with invalid page spec exits `1`
- inspect warning path for likely encrypted documents is exercised if we can add a fixture cheaply

### render tests

Until `render` exists, keep a minimal stub test asserting:

- command is present
- current "not implemented" behavior stays explicit

Once `render` is implemented, replace the stub test with real artifact and exit-code tests.

## fixture strategy

Use a small fixture set with explicit roles:

- `sample.pdf` as the baseline happy-path digital PDF
- a multi-file temp-dir setup for batch and expansion tests
- a tiny invalid PDF byte file for parse-failure tests
- later, an encrypted PDF fixture only if we can add one cheaply and legally
- a tiny subset of `../pdf-parser-benchmark/corpus` or synthetic fixtures for slow benchmark-compatibility tests

Fixture principles:

- keep fixtures small for fast local iteration
- prefer fixtures whose assertions are structural, not prose-heavy
- avoid overfitting to one exact extracted paragraph unless the test is specifically about text normalization
- for slow benchmark-facing tests, prefer the benchmark repo's smallest synthetic fixtures over large business or academic documents
- do not assume benchmark corpus files are present in every local checkout; slow tests need a clean skip path or a tiny fallback fixture

## Rust test layout

We want two visible directories:

- `tests/fast`
- `tests/slow`

Important implementation note:

- Cargo does not automatically treat nested files under `tests/` as standalone integration-test crates

So the layout should be one of these:

1. Keep top-level harness files such as `tests/fast.rs` and `tests/slow.rs` that `mod` files from `tests/fast/` and `tests/slow/`.
2. Or declare explicit `[[test]]` entries in `Cargo.toml` for dedicated harness files that then include modules from those directories.

Recommendation:

- use `tests/fast.rs` and `tests/slow.rs` as lightweight harness entrypoints
- keep the actual test files grouped underneath `tests/fast/` and `tests/slow/`
- make `tests/slow.rs` explicitly opt-in during routine development
- migrate the current top-level integration tests into `tests/fast/` so every integration test clearly belongs to either the fast or slow tier

Recommended gating mechanisms for `tests/slow.rs`:

- mark slow tests `#[ignore]` and document the explicit run command
- or require an env var such as `RPDF_RUN_SLOW=1` and skip cleanly when it is absent
- or combine both for extra safety

## proposed test file layout

Keep integration tests grouped first by speed tier, then by contract area rather than by internal module:

- `tests/fast.rs` as the fast integration-test harness
- `tests/slow.rs` as the slow integration-test harness
- `tests/fast/cli_root.rs` for root help and version behavior
- `tests/fast/parse_flag_matrix.rs` for invalid/valid CLI combinations
- `tests/fast/parse_outputs.rs` for markdown output-path rules and overwrite behavior
- `tests/fast/parse_json_contract.rs` for JSON structure and config echoing
- `tests/fast/parse_stdout_stderr.rs` for shell hygiene
- `tests/fast/parse_batch_contract.rs` for batch semantics and per-stem outputs
- `tests/fast/parse_status_codes.rs` for success, partial success, usage error, and failure
- `tests/fast/inspect_contract.rs` for inspect behavior
- `tests/fast/render_contract.rs` for stub behavior now and real behavior later
- `tests/slow/benchmark_adapter_smoke.rs` for benchmark-style parse contract checks
- `tests/slow/benchmark_sequential.rs` for repeated parse behavior against a shared output directory
- `tests/slow/benchmark_failure_surface.rs` for benchmark-visible failure propagation

We do not need to rename existing files immediately, but new coverage should move toward this grouping so the suite stays readable.

Migration note:

- move the current top-level integration tests into `tests/fast/`
- keep their coverage intact while renaming or regrouping by contract area
- do not leave long-lived integration tests at the top level once the new harness structure exists

For execution ergonomics:

- fast tests should be the default
- slow tests should be easy to run separately when touching benchmark-facing code paths
- document the exact commands once the harness layout is implemented
- the default `cargo test` path should not execute slow tests unless the explicit slow-test gate is enabled

## implementation order

### phase 1: finish pure contract unit tests

Start with the cheap, stable unit tests:

- validation matrix
- config normalization
- exit-code mapping
- overwrite helpers

### phase 2: fill the biggest integration gaps

Add integration tests for:

- overwrite protection
- stdout/stderr hygiene
- JSON required-field contract
- invalid mode values

### phase 3: add slow benchmark-compatibility smoke tests

Add a minimal slow suite that interacts with `../pdf-parser-benchmark/`:

- one basic parse-success smoke test
- one sequential multi-document smoke test
- one failure-propagation smoke test

Exit criterion:

- we can prove `rpdf` satisfies the benchmark-side parser contract on a tiny representative slice without running the whole benchmark

### phase 3a: add a real benchmark-side adapter

Once the emulated slow smoke tests are useful:

- add a thin `rpdf` adapter in `../pdf-parser-benchmark/` that implements the benchmark's `BaseParser` contract
- add at least one slow test that calls that real adapter instead of only emulating the contract from the `rpdf` repo side

Exit criterion:

- benchmark-facing compatibility is proven through the real adapter surface, not only through a local emulation helper

### phase 4: add determinism checks

Once the core contract suite is green, add:

- repeated-run stdout equality
- repeated-run JSON stability checks on stable fields

### phase 5: refactor the suite for maintainability

Only after the key coverage exists:

- consolidate overlapping tests
- group files by contract area
- extract small test helpers for temp dirs, fixture lookup, and JSON loading

## definition of done

This contract-eval plan is complete when:

- every documented v1 CLI rule has either a unit test or an integration test
- shell-visible behavior is covered by integration tests
- pure validation and mapping logic is covered by unit tests
- the suite clearly distinguishes exit `1`, `2`, and `3`
- JSON required fields and basic shape are locked down
- stdout/stderr cleanliness is protected
- `rpdf` has a small slow smoke suite proving compatibility with `../pdf-parser-benchmark/`
- tests are fast enough to run routinely during local iteration

## what not to do yet

Do not spend early effort on:

- exact Markdown golden files for complex PDFs
- parser-quality scoring for headings, lists, or tables
- large benchmark corpora inside the normal test suite
- brittle assertions on incidental clap help formatting
- broad snapshot tests of whole JSON blobs when only a few fields matter

Those belong in later parser-quality eval work, not this contract-preservation layer.
