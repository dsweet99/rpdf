# rpdf application plan

## purpose

Build `rpdf` as a local-first PDF parser application that turns PDFs into high-quality structured output for:

- Markdown and text extraction
- RAG and LLM pipelines
- element-level JSON with page coordinates
- debugging and parser evaluation
- benchmarking against `pdf-parser-benchmark`

The application should be useful on its own as a CLI, while also being easy to wrap from other tools.

## product goals

### v1 goals

- Parse digital PDFs locally with no cloud dependency.
- Produce deterministic output for the same input and same options.
- Preserve reading order better than plain text dump tools.
- Emit a canonical structured JSON format and derive Markdown from it.
- Expose a stable CLI that is easy to script and benchmark.
- Handle common business and academic PDFs well enough to benchmark honestly.
- Make failures legible with warnings, partial-success reporting, and stable exit codes.

### stretch goals

- OCR support for scanned PDFs.
- optional hybrid mode for hard pages such as borderless tables, formulas, or poor scans
- visual debugging output
- Tagged PDF and structure-tree awareness
- library API after the CLI is stable

### non-goals for v1

- perfect OCR
- full PDF editing
- PDF/UA remediation
- proprietary model dependencies
- solving every invoice and scientific-paper edge case in the first milestone

## advice from nearby repos

This plan is informed by the sibling repos in `/home/dsweet/Projects/pdfs/`.

### from `opendataloader-pdf`

Useful lessons from:

- `/home/dsweet/Projects/pdfs/opendataloader-pdf/README.md`
- `/home/dsweet/Projects/pdfs/opendataloader-pdf/options.json`
- `/home/dsweet/Projects/pdfs/opendataloader-pdf/docs/hybrid/hybrid-mode-design.md`

What to copy:

- Make the CLI batch-friendly. That repo repeatedly emphasizes that per-invocation startup is expensive, so users should process multiple files in one call when possible.
- Make JSON the canonical output model and Markdown a rendered view of that model.
- Include bounding boxes, page numbers, stable element ids, and semantic element types.
- Treat reading order as a first-class problem, not an afterthought.
- Prefer a simple local mode by default, with optional heavier or hybrid paths later.
- Expose operational flags such as `--pages`, `--use-struct-tree`, output formatting controls, and quiet mode.
- Be explicit about partial success instead of collapsing every imperfect run into pass or fail.

What not to copy too early:

- a large cross-language wrapper surface
- a hybrid server before the local core is trustworthy
- a very broad option matrix before the core output model stabilizes

### from `pypdfium2`

Useful lessons from:

- `/home/dsweet/Projects/pdfs/pypdfium2/README.md`
- `/home/dsweet/Projects/pdfs/pypdfium2/src/pypdfium2/_helpers/textpage.py`

What to copy:

- Use PDFium as a strong low-level engine for opening PDFs, rendering pages, and extracting text geometry.
- Normalize CRLF to LF in emitted text.
- Prefer bounded-text extraction or object-aware extraction over naive range slicing.
- Use character boxes, text rectangles, and page objects as raw signals for a higher-level layout engine.
- Pin the PDFium version and test against real PDFs because ABI and extraction behavior can shift.

Critical limitation to plan around:

- PDFium does not do layout analysis for you. It gives text and geometry, but `rpdf` must supply its own block building, reading-order heuristics, table inference, and element classification.

Operational constraint:

- PDFium is not thread-safe. `rpdf` should assume one-document-at-a-time processing within a process and use process-level parallelism later if needed.

### from `pdf-parser-benchmark`

Useful lessons from:

- `/home/dsweet/Projects/pdfs/pdf-parser-benchmark/README.md`

What to copy:

- Measure text quality, structure quality, and table quality separately.
- Benchmark by domain because rankings shift dramatically by document type.
- Design for clean CLI invocation and deterministic Markdown output.

Constraint to remember:

- The benchmark currently uses Python adapters internally, but `rpdf` itself should still be a standalone CLI. The benchmark-side wrapper can stay thin.

## product shape

`rpdf` should be a standalone executable first.

Primary command:

```bash
rpdf parse input.pdf --output output.md
```

Required support commands:

```bash
rpdf --version
rpdf parse input.pdf --stdout
rpdf parse input.pdf --json output.json
rpdf inspect input.pdf
```

Recommended batch shape:

```bash
rpdf parse file1.pdf file2.pdf dir/ --output-dir out/
```

## CLI design

### command surface

#### `rpdf parse`

Core parser command.

Examples:

```bash
rpdf parse input.pdf --output output.md
rpdf parse input.pdf --json output.json
rpdf parse input.pdf --output output.md --json output.json
rpdf parse input.pdf --stdout
rpdf parse file1.pdf file2.pdf dir/ --output-dir out/
```

Flags for v1:

- `--output <path>`: write Markdown to a file
- `--json <path>`: write structured JSON to a file
- `--stdout`: write Markdown to stdout
- `--output-dir <dir>`: directory mode for batch runs
- `--pages <spec>`: page filter such as `1,3,5-7`
- `--password <value>`: password for encrypted PDFs
- `--use-struct-tree`: prefer tagged-PDF structure when present
- `--reading-order <off|basic|xycut>`: control layout heuristics
- `--table-mode <off|lines|heuristic>`: table extraction mode
- `--include-header-footer`: keep detected headers and footers
- `--keep-line-breaks`: preserve more original line structure
- `--quiet`: reduce logs
- `--debug-json <path>`: write extra diagnostic JSON

Flags for later phases:

- `--ocr`
- `--ocr-lang <langs>`
- `--render-dpi <n>`
- `--hybrid <backend>`
- `--hybrid-url <url>`

CLI contract:

- stdout contains only requested document output
- stderr contains diagnostics
- exit `0` on success
- exit nonzero on failure
- if some pages fail but useful output exists, still emit output and report partial success in JSON and stderr

### CLI contract matrix

The following rules should be treated as part of the v1 CLI contract, not as implementation details.

#### input cardinality

- `rpdf parse <input.pdf>` is the canonical single-file form.
- `rpdf parse <file1> <file2> <dir/> --output-dir <dir>` is the canonical batch form.
- batch mode is allowed only when `--output-dir` is provided or when the default output location rule is unambiguous.

#### output mode rules

- `--stdout` is single-input only.
- `--stdout` is Markdown-only in v1.
- `--stdout` cannot be combined with `--output-dir`.
- `--stdout` cannot be combined with multiple inputs.
- `--output <path>` is single-input only.
- `--output <path>` may be combined with `--json <path>` for single-input runs.
- `--json <path>` is single-input only.
- batch mode writes per-input outputs beneath `--output-dir`.

#### default path rules

- for single-input runs with no explicit output flags, `rpdf` should write Markdown next to the input PDF using the same stem and `.md` extension
- when `--json` is requested without `--output`, JSON should use the caller-provided path in single-input mode
- in batch mode, default filenames should be derived from each input stem and should avoid silent overwrites

#### partial-success rules

- `partial_success` means at least one requested page or one requested document failed, but at least one useful output artifact was produced
- single-input partial success should exit with code `3`
- batch mode should still complete all independent inputs even if some inputs fail
- batch mode should emit a per-input status summary to stderr
- if every requested input fails and no useful output is produced, the run is `failure`, not `partial_success`

#### shell-safety rules

- stdout must remain machine-clean with no progress bars or log lines
- stderr may contain warnings, summaries, and diagnostics
- exit codes must be stable enough for scripts and benchmark wrappers
- invalid flag combinations should fail fast with exit code `1`

#### `rpdf inspect`

Quick diagnostic command for parser development.

Examples:

```bash
rpdf inspect input.pdf
rpdf inspect input.pdf --pages 1-2
```

Outputs:

- page count
- encryption and tagging info
- whether a text layer exists
- basic object counts
- likely parse strategy
- warnings about unsupported or suspicious features

#### `rpdf render`

Optional but valuable for development.

Examples:

```bash
rpdf render input.pdf --page 1 --output page1.png
```

This is mainly for debugging OCR, bounding boxes, and table heuristics.

## canonical output model

The canonical representation should be JSON, not Markdown.

Suggested top-level shape:

```json
{
  "schema_version": "1.0",
  "parser_version": "0.1.0",
  "status": "success",
  "input": "input.pdf",
  "page_count": 3,
  "warnings": [],
  "failed_pages": [],
  "config": {
    "reading_order": "basic",
    "table_mode": "lines"
  },
  "pages": [
    {
      "page": 1,
      "width": 612.0,
      "height": 792.0,
      "elements": [
        {
          "id": "p1-e1",
          "type": "heading",
          "bbox": [72.0, 700.0, 540.0, 730.0],
          "text": "Introduction",
          "heading_level": 1
        }
      ]
    }
  ]
}
```

Element fields to standardize early:

- `id`
- `type`
- `page`
- `bbox`
- `text`
- `children`
- optional style hints such as font size, bold, italic, list level, table metadata

Core element types:

- `heading`
- `paragraph`
- `list`
- `list_item`
- `table`
- `table_row`
- `table_cell`
- `code_block`
- `quote`
- `image`
- `caption`
- `footnote`
- `formula`

Design rule:

- Markdown is rendered from the element tree.
- Plain text is rendered from the same tree.
- Debug and evaluation tools should prefer JSON.

### v1 schema contract

The v1 JSON model should be documented as a versioned contract, even if the first release starts with a hand-written schema document before a full JSON Schema file exists.

#### required top-level fields

- `schema_version`: schema contract version such as `1.0`
- `parser_version`: `rpdf` application version
- `status`: `success`, `partial_success`, or `failure`
- `input`: original input path or source identifier
- `page_count`
- `warnings`
- `failed_pages`
- `config`: normalized parse options that materially affect output
- `pages`

#### required page fields

- `page`
- `width`
- `height`
- `elements`

#### required element fields

Every element must have:

- `id`
- `type`
- `page`
- `bbox`

Text-bearing elements must also have:

- `text`

Nested elements may additionally have:

- `children`

#### required type-specific minimums

- `heading`: `text`, `heading_level`
- `paragraph`: `text`
- `list`: `children`
- `list_item`: `children` or `text`
- `table`: `children` or explicit row/cell structure
- `table_cell`: positional metadata plus `children` or `text`
- `image`: `bbox`
- `caption`: `text`
- `footnote`: `text`
- `formula`: `text`

#### provenance and reproducibility fields

The v1 schema should carry enough metadata to reproduce benchmark results and golden tests:

- parser version
- selected parse options
- PDF engine version
- optional build or feature flags if they materially affect output

#### compatibility policy

- additive fields are allowed in minor schema revisions
- renaming or removing required fields requires a major schema version change
- Markdown rendering should be treated as a derived format, not the compatibility anchor

#### confidence policy

V1 does not need per-element confidence scores.

If confidence is added later:

- absence of confidence must remain valid for older outputs
- confidence must not silently change rendering semantics without an explicit config flag

#### hidden-content policy

If `rpdf` suppresses hidden, off-page, or suspicious text, the JSON should preserve that fact through warnings or debug artifacts rather than making such drops invisible.

## parsing architecture

### phase 1: document ingestion

- open PDF
- detect encryption
- detect page count
- detect whether a structure tree exists
- detect whether a usable text layer exists
- initialize PDFium-backed handles

### phase 2: raw extraction

For each page:

- extract page bounds
- extract text spans or bounded text
- extract character boxes
- extract text rectangles
- extract page objects such as text, images, and vector hints

### phase 3: normalization

- normalize Unicode
- normalize line endings from CRLF to LF
- replace invalid characters deterministically
- remove or flag clearly hidden or off-page text
- record warnings instead of silently dropping surprising cases

### phase 4: page segmentation

Build intermediate structures:

- glyphs
- words
- lines
- blocks

This layer is where `rpdf` earns its value. PDFium gives geometry; `rpdf` must assemble logical content.

### phase 5: reading order

Reading order should be configurable but deterministic.

Suggested modes:

- `off`: preserve source-local order as a debugging baseline
- `basic`: simple top-to-bottom, left-to-right heuristics
- `xycut`: more advanced block ordering for columns and mixed layouts

Initial strategy:

- start with `basic`
- build `xycut` once block segmentation is stable
- use struct-tree ordering when `--use-struct-tree` is enabled and trustworthy

### phase 6: semantic classification

Classify blocks into:

- headings
- paragraphs
- lists
- tables
- code blocks
- captions
- footnotes

Signals:

- font size
- font weight
- indentation
- alignment
- bullet and numbering patterns
- ruling lines
- whitespace gaps
- column boundaries
- repetition across pages for header/footer detection

### phase 7: table extraction

Table extraction should be its own subsystem, not buried inside paragraph logic.

V1 strategy:

- detect border-based tables
- detect aligned text grids without borders when obvious
- produce a structured table model even if Markdown rendering is imperfect

Important product choice:

- prefer correct table structure in JSON first
- render Markdown tables only when confidence is high
- otherwise degrade gracefully with warnings rather than hallucinating a clean table

### phase 8: rendering

Render outputs from the canonical element tree:

- Markdown
- plain text
- JSON

Rendering goals:

- stable output across runs
- no incidental whitespace churn
- preserve heading hierarchy
- preserve lists and tables when confidence allows

## architecture modules

Suggested Rust crate layout:

- `rpdf-cli`: argument parsing and process exit behavior
- `rpdf-core`: orchestration, types, configuration
- `rpdf-pdfium`: PDFium bindings adapter layer
- `rpdf-layout`: words, lines, blocks, reading order
- `rpdf-structure`: headings, lists, tables, captions, footnotes
- `rpdf-render`: Markdown, text, JSON emitters
- `rpdf-debug`: debug dumps, overlay data, diagnostics
- `rpdf-ocr`: optional future OCR support

Key design principle:

- keep PDFium-specific code behind a small boundary
- keep parsing logic engine-agnostic where possible

## failure model

`rpdf` should explicitly model:

- `success`
- `partial_success`
- `failure`

Partial success cases:

- one page fails to classify
- one page is encrypted or corrupted while others load
- table extraction fails but text extraction succeeds
- OCR fallback unavailable for scanned pages

Expected behavior:

- JSON includes `status`, `warnings`, and `failed_pages`
- CLI prints a concise summary to stderr
- exit code policy remains predictable

Suggested exit codes:

- `0`: success
- `1`: usage or configuration error
- `2`: parse failure with no usable output
- `3`: partial success

## benchmark strategy

### external benchmark compatibility

For `pdf-parser-benchmark`, the important interface remains:

```bash
rpdf parse input.pdf --output output.md
rpdf --version
```

Even if the benchmark keeps a Python adapter internally, `rpdf` should not depend on Python.

### internal evaluation loop

Before chasing leaderboard results, `rpdf` should maintain its own regression suite across:

- legal and contract PDFs
- invoices and table-heavy documents
- HR and resume-style documents
- multi-column articles
- tagged PDFs
- rotated pages
- image-heavy and scanned PDFs

Metrics to track:

- exact and normalized text diff
- reading-order error cases
- heading precision and recall
- list preservation
- table structure accuracy
- parse time per page
- crash-free rate

### evaluation philosophy

- optimize by document class, not only by overall score
- separate text, structure, and table quality
- keep a small fast corpus for iteration and a larger corpus for release gates

## implementation roadmap

### milestone 0: repo bootstrap

- create Cargo workspace
- choose PDFium binding strategy
- define core types
- implement `rpdf --version`
- implement `rpdf inspect input.pdf`

Exit criterion:

- can open PDFs, report metadata, and fail cleanly

### milestone 1: raw text and JSON skeleton

- open pages
- extract raw text and geometry
- normalize Unicode and line endings
- emit basic page JSON with raw spans

Exit criterion:

- can produce structured debug JSON for real PDFs

### milestone 2: block building and basic Markdown

- build words, lines, and blocks
- implement basic reading order
- emit paragraphs and headings
- render Markdown and text

Exit criterion:

- useful Markdown on simple digital PDFs

### milestone 3: lists, headers/footers, and tables

- detect lists
- detect repeated headers and footers
- implement first table detector
- improve Markdown stability

Exit criterion:

- good results on synthetic and business PDFs with moderate structure

### milestone 4: benchmark readiness

- lock CLI interface
- add deterministic output tests
- compare against nearby benchmark corpora
- tune for `pdf-parser-benchmark` compatibility

Exit criterion:

- `rpdf parse input.pdf --output out.md` is stable enough to benchmark

### milestone 5: advanced structure

- add `--use-struct-tree`
- improve multi-column ordering
- improve caption and footnote handling
- add richer JSON metadata

### milestone 6: OCR or hybrid path

- add optional OCR module for image-only PDFs
- optionally add hybrid routing for complex pages
- preserve the local deterministic path as default

## technical choices to make early

### 1. PDF engine

Current best direction:

- use PDFium for low-level extraction and rendering

Reason:

- strong low-level capabilities
- liberal licensing story compared with some alternatives
- good geometry support
- good rendering support for future OCR fallback

Risk:

- PDFium does not provide layout analysis
- PDFium version changes can alter behavior

Mitigation:

- pin versions
- keep golden tests
- isolate engine-specific code

### 1a. PDFium packaging and version policy

This decision should be locked before deep parser work begins.

#### v1 default policy

- ship `rpdf` as a standalone Rust CLI with a pinned PDFium dependency strategy
- prefer vendored or otherwise reproducible PDFium builds for official releases
- treat system PDFium as an advanced or developer-oriented option, not the primary release path

#### why

- nearby PDFium tooling shows that ABI and version mismatches are a real engineering concern
- reproducible benchmark results depend on stable engine behavior
- users should not need to solve PDFium discovery before trying `rpdf`

#### explicit decision points

The implementation must choose and document:

1. whether official binaries bundle PDFium or dynamically link to a pinned system package
2. which PDFium version is pinned for the first release
3. whether XFA or similar optional features are enabled
4. how Linux, macOS, and Windows builds obtain matching binaries

#### acceptance criteria

- `rpdf --version` should report both `rpdf` version and PDF engine version in a machine-readable or clearly parseable way
- CI must exercise the pinned PDFium configuration used for official releases
- golden-output tests must run against the same engine configuration used in release artifacts
- unsupported local engine overrides must be clearly labeled as such

#### fallback policy

- developer builds may support a custom or system PDFium override
- release notes and bug reports should always capture which PDFium build was used
- any mode that allows engine overrides should be excluded from the default reproducibility story

### 2. canonical representation

Current best direction:

- JSON element tree as the source of truth

Reason:

- supports Markdown, text, debug tooling, and RAG
- easier to test than Markdown-only output
- aligns with the strongest neighboring design in `opendataloader-pdf`

### 3. concurrency model

Current best direction:

- no shared-document multithreading in core parsing
- add process-level document parallelism later

Reason:

- safer with PDFium
- easier to debug

### 4. OCR timing

Hypothesis:

Adding OCR too early may slow progress and blur the quality bar for the digital-PDF core.

Predictions:

- early OCR work will increase complexity before line, block, and table logic stabilize
- digital-PDF quality will improve faster if OCR is deferred

Test:

- first benchmark only digital and text-layer PDFs
- add scanned corpus only after stable Markdown and JSON on digital PDFs

Confounders:

- if the target users primarily care about scans, the product priorities may need to change

## developer ergonomics

`rpdf` should be pleasant to debug.

Add from early stages:

- `--debug-json`
- `inspect` command
- optional per-page dumps
- stable warnings
- small fixture corpus checked into the repo if licensing allows

Helpful later:

- render page overlays with detected blocks, tables, and reading order
- compare JSON output between versions

## release criteria for first public version

The first public version should not wait for perfection. It should ship when:

- the CLI is stable
- Markdown output is useful on simple and moderate digital PDFs
- JSON output is coherent and documented
- failures are explicit
- there is a benchmark story
- there is a small regression corpus

## open questions

1. Should `rpdf` be a single binary only, or should it also expose a Rust library API in v1?
[dsweet] single binary
2. Should `xycut` be implemented in v1 or staged behind `basic` reading order first?
[dsweet] staged
3. How aggressive should header/footer stripping be by default?
[dsweet] idk
4. Should Markdown tables be emitted only when confidence is high, with JSON carrying richer structure?
[dsweet] yes
5. Is OCR important enough for the first public milestone, or should it wait until after benchmark readiness?
[dsweet] wait

## current recommendation

Build `rpdf` as:

- a standalone Rust CLI
- powered by PDFium for low-level extraction
- with a canonical JSON element tree
- with Markdown generated from that tree
- focused first on digital PDFs
- benchmark-friendly through a stable `parse` command

Do not start with:

- Python bindings
- a multi-language SDK
- hybrid backends
- OCR-first complexity

The fastest route to a strong application is:

1. stable CLI
2. trustworthy JSON schema
3. reading-order and block heuristics
4. table extraction
5. benchmark feedback
6. optional OCR and hybrid expansion
