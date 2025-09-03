# Scene Validator (Hop 7)

Validate Godot .tscn files for broken scripts/resources with CI-friendly outputs.

What it checks
- Scripts
  - Missing script="res://..." files
  - script = ExtResource("id") mapping to [ext_resource] and missing file
  - Unknown ExtResource ids
- Properties
  - Generic property = ExtResource("id") (e.g., texture, mesh) missing file
  - Unknown ExtResource ids
- SubResource
  - Tracks [sub_resource] ids; flags unknown SubResource("id") usages anywhere on a line
- preload/load
  - preload("res://...") and load("res://...") missing file detection
- Determinism
  - Findings sorted deterministically for stable CI

CLI usage
- Validate scenes and print JSON
  - cargo run -p godot --bin godot-analyzer -- --root . --validate_scenes --json
- Export SARIF and JUnit
  - cargo run -p godot --bin godot-analyzer -- --root . --validate_scenes --sarif-out godot.sarif --junit-out godot.junit.xml
- Select checks (defaults to all)
  - --scene-check script | properties | subresource | preload | load
  - Example: --scene-check preload --scene-check load

VS Code tasks
- scene validate (JSON)
- scene validate (SARIF+JUnit)

Outputs
- SARIF
  - ruleId: scene-validator for scene findings; godot-analyzer for others
  - Driver rules metadata included for both ruleIds
- JUnit
  - classname: scene-validator for scene findings

Example messages
- Missing script: res://scripts/player.gd
- Script ExtResource(1) missing file res://scripts/player.gd
- Unknown ExtResource id: 9
- Property 'texture' ExtResource(3) missing file res://assets/tex.png
- Unknown SubResource id: 12
- Preload missing file: res://scripts/miss.gd
- Load missing file: res://scripts/miss.gd

Notes
- uid:// references are ignored for existence checks.
- Parsing is line-based for speed; embedded multiline scripts may require future enhancements.
