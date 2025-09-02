## ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
## ┃ File: Template.gd                                                   ┃
## ┃ Purpose: Starter template with project commenting style             ┃
## ┃ Author: EricA1019                                                   ┃
## ┃ Last Updated: 2025-09-02                                           ┃
## ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

extends Node

## describe_constant
const EXAMPLE_CONST := 42  # inline explanation

## example_method
## Short behavior summary; list edge cases and failure modes.
func example_method(arg: int) -> int:
    # ── Input Validation ────────────────────────────────────────────────
    if arg < 0:
        push_warning("negative arg")
        return 0

    # ── Core Logic ─────────────────────────────────────────────────────
    var result := arg + EXAMPLE_CONST

    # ── Post ───────────────────────────────────────────────────────────
    return result

#EOF