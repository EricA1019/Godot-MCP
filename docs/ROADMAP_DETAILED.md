# Godot MCP Server — Detailed Roadmap (Close-to-Shore)

Living roadmap of tiny, runnable hops with crisp acceptance criteria. Each hop must end green: tests pass, app boots, docs updated.

## Legend
- Status: [PLANNED] [IN-PROGRESS] [DONE]
- Effort: XS (≤1h) / S (1–2h) / M (2–4h) / L (4–8h)
- Artifacts: Code, Tests, Data/Schema, Docs

---

## Phase 1 — Foundation (Weeks 1–2)

### Hop 1: Bootstrap MCP Server Skeleton [DONE] (S)
- Goal: Create Rust workspace, MCP server crate, and tool plugin scaffolding
- Deliverables:
  - crates/mcp-server: main.rs, lib.rs; health endpoint
  - crates/common: logging, config loader
  - tasks: build + test tasks.json
- Tests:
  - Unit: config load ok, health endpoint 200
  - Smoke: server boots
- Docs:
  - DEV_LOG.md entry, PROJECT_INDEX.md section
- Acceptance:
  - `cargo test` green, server runs, logs show tags

### Hop 2: Master Index – Full-text Core [DONE] (M)
- Goal: Add Tantivy-based full-text index with FS watcher
- Deliverables:
  - crates/index: full_text_index.rs, change_monitor.rs
  - CLI: `index scan`, `index query "term"`
- Tests:
  - Unit: add/update/delete file updates index
  - Integration: search returns expected hits
  - Watcher start/stop endpoints
- Docs:
  - Index schema summary in GODOT_MCP_SPECIFICATION.md
- Acceptance:
  - Index endpoints + watcher controls exposed in MCP server
  - Update/delete test passes; queries reflect latest commit
  - Initial perf sanity: apply_batch per file < 250ms in CI env

### Hop 3: Context Bundler v1 [DONE] (S)
- Goal: Bundle top N relevant docs/code for a query using Master Index
- Deliverables:
  - crates/context: bundler.rs (rank, cap size, dedupe)
- Tests:
  - Unit: relevance ranking deterministic
  - Integration: /context/bundle endpoint smoke + kind filter + deterministic ordering + cap enforcement
- Docs: DEV_LOG.md update
- Acceptance: bundle ≤ 64KB, ranked by recency+relevance; server exposes /context/bundle; tests green

### Hop 4: Auto-Documentation v1 [DONE] (S)
- Goal: Verify/create CTS docs from templates with safe, idempotent updates
- Deliverables:
  - crates/tools (autodoc lib + CLI with clap): ensure DEV_LOG.md, PROJECT_INDEX.md, WORKFLOW_PROJECT.md
  - Region markers for managed blocks; atomic writes; JSON report
  - VS Code task; README usage
- Tests:
  - Smoke: creates or updates docs in temp dir
  - Idempotence: second run yields verified with no changes
- CI:
  - Non-blocking job to run `autodoc --check --json` and upload report (follow-up: make blocking)
- Acceptance:
  - CLI supports --root, --dry-run, --check, --json
  - Managed regions updated without clobbering custom content
  - Exit code 2 when --check finds changes
  - CI runs a non-blocking autodoc check

### Hop 5: Meta-Tagger v1 [DONE] (M)
- Goal: Scan repo, classify files, update PROJECT_INDEX.md cleanup section; add severities, ignore patterns, and CI artifact
- Deliverables:
  - crates/tools/metatagger.rs + CLI (tools/bin/metatagger.rs)
  - Classifiers: temp files, orphan .import, large files, unused images, duplicate assets, stale export presets
  - Severity levels (info|warn|error) with CLI --min-severity filter
  - Ignore patterns via .metataggerignore (globset)
  - PROJECT_INDEX.md METATAGGER-managed cleanup region with severity tags
  - CI: optional step to publish JSON report as artifact (added)
  - Docs: METATAGGER.md, examples for ignore and baseline
- Tests:
  - Smoke: temp dir classification produces ≥1 finding for seeded temp files
  - Duplicate detection: identical content across two files flagged deterministically
  - Ignore patterns: files matched by .metataggerignore are excluded
  - Integration: PROJECT_INDEX cleanup region updated deterministically with severities
- Acceptance:
  - CLI prints JSON or summary; findings sorted by severity, kind, then path
  - .metataggerignore respected; --min-severity filters output; --fail-on gates CI
  - Baseline suppression supported; SARIF/JUnit outputs available
  - CI uploads metatagger.json on push/PR (non-blocking)

---

## Phase 2 — Godot Core Tools (Weeks 3–4)

### Hop 6: Godot Project Analyzer [DONE] (M)
- Goal: Parse project.godot, addons/, export_presets.cfg; emit CI-friendly outputs
- Deliverables:
  - crates/godot: library + CLI (godot-analyzer) with JSON/ SARIF/ JUnit outputs
  - Reports engine format version, addons list, export presets (name/platform/export_path)
  - Checks: missing application icon (config/icon), missing main scene (run/main_scene), addon plugin.cfg presence, broken ext_resource paths, export_path parent dirs
- Tests:
  - Smoke test resolves repo root and detects addons
  - Unit tests for icon/main_scene warnings, SARIF/JUnit generation, missing ext_resource detection
- Acceptance:
  - Deterministic JSON fields and ordering; handles missing files gracefully
  - CLI supports --min-severity filtering and --fail-on gating; CI uploads analyzer artifacts (JSON/SARIF)
  - Optional: SARIF uploaded to GitHub Code Scanning when available

### Hop 7: Scene Validator [DONE] (M)
- Goal: Validate .tscn hierarchy, scripts, resources
- Deliverables:
  - crates/godot/scene_validate.rs: validator parses [node] paths, script attributes, and [ext_resource] declarations; supports script = ExtResource("id") mapping
  - Generic property ExtResource("id") handling (e.g., texture = ExtResource("1")) with file existence checks
  - Tracks [sub_resource] ids and flags unknown SubResource("id") references anywhere on a line
  - Detects preload("res://…") and load("res://…") missing file paths
  - scene_issues_as_report and scene_issues_as_report_with with SceneCheckOptions to enable/disable categories
  - CLI: godot-analyzer --validate_scenes with optional --scene-json-out and repeatable --scene-check <script|properties|subresource|preload|load>
  - Outputs: SARIF uses ruleId "scene-validator" for validator findings; JUnit uses classname="scene-validator"; analyzer stays "godot-analyzer"; SARIF driver rules metadata included for both
  - Deterministic ordering preserved after merging scene findings
- Tests:
  - Unit tests for missing script presence and OK path
  - Tests for ext_resource declared path missing, unknown id, and ExtResource("id") mapping
  - Test for property ExtResource("id") reporting and SARIF ruleId separation
  - Tests for unknown SubResource id detection and preload/load missing file detection
  - Future: fixtures covering bad/missing nodes, scripts
- Acceptance:
  - Issues include file:line and node path context; deterministic ordering
  - CLI flag produces merged report and optional standalone JSON; --scene-check filters which categories run
  - SARIF/JUnit reflect separate rule/class for scene validator; driver rules metadata present
  - Detects missing files referenced via scripts, ext_resources, SubResource ids, and preload/load
  - No panics on malformed but parseable scenes; graceful degradation

### Hop 8: Signal Validator + Trace [DONE] (M)
 Goal: Analyze signal definitions and connections
 Deliverables:
  - crates/godot/signal_validate.rs: parse [connection] entries; verify from/to nodes exist; require signal/method; detect duplicates
  - Target method existence checks for GDScript receivers (regex on func definitions); invalid method names flagged; C#/native skipped
  - CLI: godot-analyzer --validate_signals; merged outputs with SARIF ruleId "signal-validator" and JUnit classname="signal-validator"
  - DOT graph export across scenes via --signal-dot-out godot-signals.dot (rankdir=LR); CI renders PNG via Graphviz
  - VS Code tasks: "signal validate (JSON)", "signal validate (SARIF+JUnit)", "signal graph (DOT)", and DOT→PNG
  - Quick-fix hints embedded in messages
- Tests:
 Tests:
  - Flags unknown 'to' node and duplicate connection
  - Generates DOT graph for simple two-node cycle
  - Method existence: missing method flagged; present method passes; C# receiver skipped
 
 Acceptance:
  - Deterministic, CI-friendly outputs; no panics on malformed scenes
  - Connection integrity validated plus GDScript method existence heuristics; DOT graph emitted; PNG rendered in CI
  - Deterministic, CI-friendly outputs; no panics on malformed scenes
  - Basic connection integrity validated; future trace planned

### Hop 9: Structure Auto-Fix [DONE] (M)
- Goal: Propose+apply safe moves/renames to conventions
- Deliverables:
  - crates/godot/structure_fix.rs with dry-run planner and apply mode (moves + backups + reference rewrites for ext_resource and preload/load)
  - CLI flags: --structure_fix (plan), --structure_fix_json_out, --structure_fix_apply
  - VS Code tasks for dry-run/apply
  - CI artifact: structure-fix-plan.json on all pushes/PRs (non-blocking)
- Tests: dry-run plan stable; apply moves, creates backups, and updates references
- Acceptance: backups created prior to moves; references updated; idempotent on clean re-run

### Hop 10: GDScript Lint [IN-PROGRESS] (M)
- Goal: Lint with Godot best-practice ruleset
- Deliverables: crates/godot/script_lint.rs; CLI flag `--lint_gd`; VS Code tasks; docs/GDSCRIPT_LINT.md; CI wiring (non-blocking) and SARIF upload via analyzer
- Tests: rule fixtures for mismatch/debug prints/tabs/missing extends/missing preload path
- Acceptance: severity levels (warnings by default); CI-friendly output; deterministic ordering

---

## Phase 3 — Rusted GUTs Foundation (Weeks 5–6)

### Hop 11: GDExtension Bridge [PLANNED] (S)
- Goal: Minimal Rust GDExtension with health check in Godot
- Deliverables: crates/gdext/rusted_guts; Godot demo scene
- Tests: loads in editor; prints version
- Acceptance: Godot editor shows Rusted GUTs panel

### Hop 12: Real-time Monitor (WS) [PLANNED] (M)
- Goal: WS server in Rust + Godot client to stream scene tree deltas
- Deliverables: monitor server/client, delta protocol v1
- Tests: integration stream start/stop; backpressure
- Acceptance: < 2% frame overhead at 60 FPS

### Hop 13: Breakpoints + Variable Inspect [PLANNED] (M)
- Goal: Pause/resume, inspect node state safely
- Deliverables: bp manager, inspector API, UI
- Tests: pause on condition; resume budget < 1ms
- Acceptance: zero panics; thread-safe

### Hop 14: Memory Profiler [PLANNED] (M)
- Goal: Track allocations; leak detection
- Deliverables: alloc tracker; leak detector; report
- Tests: synthetic leaks detected; no false positives in clean run
- Acceptance: overhead < 0.1% CPU

### Hop 15: Signal Flow Tracer [PLANNED] (M)
- Goal: Live signal propagation maps
- Deliverables: tracer; DOT export; UI overlay
- Tests: multi-hop signals traced; orphan detection
- Acceptance: no GC pauses; lock-free path

---

## Phase 4 — Index Intelligence + AI (Weeks 7–8)

### Hop 16: Code Element Index [PLANNED] (M)
- Goal: Symbols, refs, calls from Rust/Python/GDScript
- Deliverables: code_index.rs; parsers + adapters
- Tests: fixtures; cross-file references
- Acceptance: search functions/classes/signals fast

### Hop 17: Search Intelligence [PLANNED] (S)
- Goal: Rank/boost by context (recency, file affinity)
- Deliverables: relevance_engine.rs
- Tests: deterministic ranking; learning updates
- Acceptance: top-5 precision ≥ 0.8 on fixtures

### Hop 18: Context Bundler + Knowledge Graph [PLANNED] (M)
- Goal: Graph from indexes; bundle by task type
- Deliverables: graph.rs; bundler v2
- Tests: bound size; path coverage
- Acceptance: bundles pass CTS context checks

### Hop 19: Tavily Integration + Protocol [PLANNED] (S)
- Goal: Implement TAVILY_PROTOCOL.md gates + client
- Deliverables: tavily_client.rs; decision engine
- Tests: mock HTTP; trigger matrix
- Acceptance: no external calls when local suffices

---

## Phase 5 — Perf + Polish (Weeks 9–10)

### Hop 20: Performance Profiler [PLANNED] (M)
- Goal: Frame timing, CPU/GPU sampling hooks
- Deliverables: profiler.rs; UI panels
- Tests: stable sampling; overhead budget respected
- Acceptance: recommendations generated

### Hop 21: Export Preset Validator [PLANNED] (S)
- Goal: Validate platform exports; suggest fixes
- Deliverables: export_validator.rs
- Tests: platform fixtures
- Acceptance: clean report for valid presets

### Hop 22: CI/CD Wiring [PLANNED] (S)
- Goal: GitHub Actions for build, test, docs
- Deliverables: workflows; badges
- Tests: workflow run green on PR
- Acceptance: artifacts uploaded (docs, reports)

### Hop 23: Docs Automation v2 [PLANNED] (S)
- Goal: Auto-update ROADMAP, DEV_LOG, PROJECT_INDEX from data
- Deliverables: doc sync tool; metadata pipeline
- Tests: idempotence; conflict-safe updates
- Acceptance: no manual edits needed for routine updates

### Hop 24: Release 0.1.0 [PLANNED] (S)
- Goal: Tag, changelog, binaries, README
- Deliverables: CHANGELOG.md; release notes
- Tests: smoke on fresh clone
- Acceptance: install + boot in under 5 minutes

---

## Risk Register & Mitigations
- GDExtension instability: pin Godot version; nightly gate
- Index scale: shard indexes; cap bundle sizes
- WS overhead: delta compression; sampling budgets
- Cross-language parsing: fall back to ctags for partial coverage

## Operating Cadence
- Daily: 1–2 hops, PR per hop, green builds
- Weekly: merge window + tag if stable
- Docs: DEV_LOG on each hop; HOP_SUMMARIES weekly

#EOF
