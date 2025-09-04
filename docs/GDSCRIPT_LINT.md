# GDScript Lint (Hop 10)

The GDScript linter scans `.gd` files and surfaces common issues early. It integrates with the Godot analyzer CLI so findings appear in JSON, SARIF, and JUnit reports with deterministic ordering.

Usage
- Add the flag `--lint_gd` to the analyzer run:
  - JSON: cargo run -p godot --bin godot-analyzer -- --root . --lint_gd --json
  - With other checks: cargo run -p godot --bin godot-analyzer -- --root . --validate_scenes --validate_signals --lint_gd --sarif-out godot.sarif --junit-out godot.junit.xml

Checks (v1)
- class_name vs filename mismatch (code: `class-name-mismatch`)
- debug prints: `print`, `prints`, `printt` (code: `debug-print`)
- tab indentation (code: `tab-indentation`)
- missing `extends` declaration (code: `missing-extends`)
- missing file targets in `preload("res://...")` / `load("res://...")` (code: `missing-resource-ref`)

Notes
- Findings are emitted as warnings by default.
- Ordering is stable to avoid CI churn.
- C# files are ignored; only `.gd` are scanned.

Suppressing rules (optional)
- Disable all rules for a file: add a top-level comment `# gd-lint: off`
- Disable specific rules: `# gd-lint: disable=debug-print,tab-indentation`

Severity (optional)
- Default severity is `warn`. Override per file:
  - `# gd-lint: level=info`
  - `# gd-lint: level=error`
