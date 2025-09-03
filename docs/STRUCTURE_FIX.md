# Structure Auto-Fix (Hop 9)

Propose and apply a conservative, deterministic project structure plan for Godot projects.

Rules (v1)
- Root-level .gd -> scripts/
- Root-level .tscn -> scenes/
- Root-level images (.png/.jpg/.jpeg) -> images/
- Skip known directories: addons/, scripts/, scenes/, images/, examples/, docs/, target/, .git/, .vscode/

CLI
- Dry-run: print JSON plan
  - cargo run -p godot --bin godot-analyzer -- --root . --structure_fix_dry_run
- Apply plan: prints rollback JSON to stdout
  - cargo run -p godot --bin godot-analyzer -- --root . --structure_fix_apply plan.json > rollback.json
- Roll back using rollback plan
  - cargo run -p godot --bin godot-analyzer -- --root . --structure_fix_rollback rollback.json

Determinism and safety
- Plan is sorted by from/to for stable output in CI
- Apply creates parent directories and fails if destination exists
- Rollback reverses in reverse order to avoid conflicts

Notes
- This is conservative v1. Future hops: recursive organization, dry-run diff formatting, and safety prompts.
