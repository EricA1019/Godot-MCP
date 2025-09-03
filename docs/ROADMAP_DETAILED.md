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

### Hop 3: Context Bundler v1 [IN-PROGRESS] (S)
- Goal: Bundle top N relevant docs/code for a query using Master Index
- Deliverables:
  - crates/context: bundler.rs (rank, cap size, dedupe)
- Tests:
  - Unit: relevance ranking deterministic
  - Integration: returns bounded bundle size
- Docs: DEV_LOG.md update
- Acceptance: bundle ≤ 64KB, ranked by recency+relevance

### Hop 4: Auto-Documentation v1 [PLANNED] (S)
- Goal: Verify/create CTS docs from templates
- Deliverables:
  - crates/tools/autodoc.rs + templates/
- Tests: unit for creation/skip when exists
- Docs: README section
- Acceptance: creates missing docs; idempotent

### Hop 5: Meta-Tagger v1 [PLANNED] (S)
- Goal: Scan repo, classify files, update PROJECT_INDEX.md cleanup section
- Deliverables: crates/tools/metatagger.rs
- Tests: classification unit tests; index update integration
- Acceptance: shows cleanup candidates deterministically

---

## Phase 2 — Godot Core Tools (Weeks 3–4)

### Hop 6: Godot Project Analyzer [PLANNED] (M)
- Goal: Parse project.godot, addons/, export_presets.cfg
- Deliverables: crates/godot/analyzer.rs
- Tests: fixture projects with expected reports
- Acceptance: JSON report with warnings+fixups

### Hop 7: Scene Validator [PLANNED] (M)
- Goal: Validate .tscn hierarchy, scripts, resources
- Deliverables: crates/godot/scene_validate.rs
- Tests: fixtures covering bad/missing nodes, scripts
- Acceptance: report maps to file:line and node paths

### Hop 8: Signal Validator + Trace [PLANNED] (M)
- Goal: Analyze signal definitions and connections
- Deliverables: crates/godot/signal_validate.rs
- Tests: orphaned, duplicate, slow handlers
- Acceptance: emits DOT graph and quick fixes

### Hop 9: Structure Auto-Fix [PLANNED] (M)
- Goal: Propose+apply safe moves/renames to conventions
- Deliverables: crates/godot/structure_fix.rs (dry-run + apply)
- Tests: dry-run diff stable; apply reversible
- Acceptance: rollback plan generated before apply

### Hop 10: GDScript Lint [PLANNED] (M)
- Goal: Lint with Godot best-practice ruleset
- Deliverables: crates/godot/script_lint.rs
- Tests: rule fixtures; suppression tags
- Acceptance: severity levels; CI-friendly output

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
