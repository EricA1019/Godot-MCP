# Structure Auto-Fix (Hop 9)

Dry-run planner to propose safe moves/renames to match project conventions.

Rules (v1)
- .gd => res://scripts/<filename>
- .tscn => res://scenes/<filename>
- Common assets (png,jpg,webp,svg,ogg,wav,mp3,ttf,otf,gdshader,tres) => res://assets/<relpath>
- Skips: addons/, crates/, docs/, target/, .git/, *.import sidecars

Safety
- Dry-run shows a deterministic JSON plan
- Apply mode creates backups under .structure_fix/backup before moving

CLI
- Plan (dry-run):
	- cargo run -p godot --bin godot-analyzer -- --root . --structure_fix
- Output plan to file:
	- cargo run -p godot --bin godot-analyzer -- --root . --structure_fix --structure_fix_json_out plan.json
- Apply plan immediately (plan is computed internally):
	- cargo run -p godot --bin godot-analyzer -- --root . --structure_fix_apply

JSON schema (v1)
- { rules: string[], moves: [{from,to}] , renames: [], edits: [], skipped: string[], stats: { scanned, proposed } }

Next
- VS Code tasks and CI dry-run artifact

