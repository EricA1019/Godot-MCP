# Signal Validator (Hop 8)

Validate [connection] entries in .tscn files.

Checks
- from/to node paths exist in the scene
- signal and method fields present
- duplicate connection detection (signal/from/to/method)
- target method existence on the receiver node's script (GDScript only)

CLI
- Include connection checks in outputs:
  - cargo run -p godot --bin godot-analyzer -- --root . --validate_signals --json
- Export a DOT graph of connections (across all .tscn under root):
  - cargo run -p godot --bin godot-analyzer -- --root . --signal-dot-out godot-signals.dot
  - PNG (optional): use Graphviz — `dot -Tpng godot-signals.dot -o godot-signals.png` (VS Code task available)
- Outputs are merged; SARIF ruleId and JUnit classname are `signal-validator` for these findings.

Issues & messages
- Unknown connection 'from' node: <path>
- Unknown connection 'to' node: <path>
- Connection missing signal field
- Connection missing method field
- Duplicate connection: signal=<s> from=<f> to=<t> method=<m>
- Invalid method name: '<name>'
- Target method not found: method='<m>' to='<node>' — define `func <m>(...)` in the target node's GDScript

Notes
- Deterministic ordering; resilient to partial/malformed scenes
- DOT graph uses composite node ids "<scene>:<node>" and rankdir=LR for readability
- GDScript method check is heuristic (regex on `func name(...)`); dynamic dispatch isn’t parsed. C# and native scripts are skipped.
- Future: quick-fix suggestions

CI
- Workflow uploads JSON/SARIF/JUnit/DOT and a rendered PNG (if Graphviz is available). SARIF is published to Code Scanning.
