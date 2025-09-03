# Metatagger

Repo hygiene scanner with deterministic findings and doc integration.

Usage
- JSON: `cargo run -p tools --bin metatagger -- --json --root .`
- Filter severity: `--min-severity warn`
- CI gate: `--fail-on warn`
- Outputs: `--sarif-out metatagger.sarif` and/or `--junit-out metatagger.junit.xml`

Ignore patterns
- Add `.metataggerignore` with glob patterns (like `.gitignore`). Example:
  - `ignored/**`
  - `**/*.large.tmp`

Baseline suppression
- Create `.metatagger.baseline.json` with entries to suppress existing issues:
```
[
  { "kind": "temp", "path": "foo.tmp" }
]
```

Determinism
- Sorted by severity, kind, then path.
- PROJECT_INDEX cleanup section gets updated with severity tags.
