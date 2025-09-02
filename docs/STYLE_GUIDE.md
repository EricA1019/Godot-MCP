# Style Guide (System-Agnostic)

A reference for consistent code structure, testing, and logging across projects.

## Core Values
- Lots of prints: verbose, tagged console output to trace flow quickly
- Data-driven first: content in resources/JSON/DB; systems discover via folder scans
- Avoid hard-coding: prefer tables, maps, registries over branches
- Auto-populating UI: containers read data/state and spawn controls
- Short hops, always green: passing tests and a bootable scene each hop

## Logging Style
- Bracketed tags per subsystem: [AbilityReg], [BuffReg], [StatusReg], [TurnMgr], [CombatMgr], [UI], [Entity]
- Concise messages with parameters
- Never silently fail: use warnings/errors for exceptional conditions
- TODO markers inline: # TODO(tag): explanation

### Logging Helper Template
```
# Generic logging helper (adapt to your language)
static func log_tagged(tag: String, message: String, args: Array = []):
    print("[", tag, "] ", message, " ", args.join(" "))
```

## Data-Driven Conventions
- Use data files (JSON/Resources/DB) for gameplay-visible content
- Names/IDs as keys for registries; authoritative source in data
- Recursive folder scans under data roots; skip hidden files
- Tables over branches; dictionary maps for type→effect

## UI Principles
- Structured containers drive everything
- Public API only on UI nodes: populate, clear, bind, show_turn, update_hp
- Zero hard-wired asset paths (allow safe fallbacks)

## Architecture Patterns
- Singletons/autoloads for shared registries and event bus
- Managers as nodes/services; prefer signals over lookups
- IDs only when needed; resolve on use

## Testing Habits
- Every feature has a test_* beside it in tests folders
- Public API only; no private peeking
- Leak hygiene where applicable; assert no new orphans
- Integration smoke tests for vertical slices

## Code Style Quick List
- End files with #EOF comment
- Functions ≤ ~40 lines; split when growing
- Guard clauses; return early
- Typing: explicit where it aids clarity
- Error handling: assert for programmer errors; warnings for recoverable data issues; errors for critical failures

## Commit Messages
- Format: feat(ui): initiative bar populates from turn order
- Types: fix, test, refactor, perf, chore

## File Organization
- Keep related tests near implementation
- Use clear, descriptive file/folder names
- Separate concerns: data, logic, presentation

#EOF

## Commenting Style (Required)

All code files must follow this structured commenting style (modeled after TagRegistry.gd):

1) File Header (top of file)
```
## ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
## ┃ File: <path/filename>                                              ┃
## ┃ Purpose: <what this file does / why it exists>                     ┃
## ┃ Author: <name/handle>                                              ┃
## ┃ Last Updated: YYYY-MM-DD                                           ┃
## ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```
Use language-appropriate comment markers for non-GDScript (// for Rust, # for Python).

2) Section Dividers
- Use Unicode box characters or "──" dividers to separate: Imports, Constants, Public API, Internal, Tests

3) Inline Comments
- Explain constants, configuration, and non-obvious logic inline

4) Function Docstrings
- Preface with `##` in GDScript or language docstrings
- Include: behavior summary, parameters, return type, edge cases, failure modes

5) Debug Prints
- Include a brief comment on intent of debug print; use tags like `[Examples][UI] ...`

6) In-Function Micro Dividers
- Use short dividers to separate stages: Input Validation, Core Logic, Post

This style is required across all projects to ensure readability and tooling consistency.

#EOF
