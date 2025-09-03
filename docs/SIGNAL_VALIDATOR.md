# Signal Validator (Hop 8)

Validate [connection] entries in .tscn files.

Checks
- from/to node paths exist in the scene
- signal and method fields present
- duplicate connection detection (signal/from/to/method)

CLI
- Include connection checks in outputs:
  - cargo run -p godot --bin godot-analyzer -- --root . --validate_signals --json
- Outputs are merged; SARIF ruleId and JUnit classname are `signal-validator` for these findings.

Notes
- Deterministic ordering; resilient to partial/malformed scenes
- Future: DOT graph export and quick-fix suggestions
