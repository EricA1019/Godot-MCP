# ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
# ┃ File: template.py                                                   ┃
# ┃ Purpose: Starter Python module aligned with project commenting style ┃
# ┃ Author: EricA1019                                                   ┃
# ┃ Last Updated: 2025-09-02                                           ┃
# ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

"""Module summary: behavior and edge cases.

Functions:
- add_const(x): returns x + EXAMPLE_CONST
"""

# ── Constants ──────────────────────────────────────────────────────────
EXAMPLE_CONST: int = 42  # inline explanation


def add_const(x: int) -> int:
    """Add EXAMPLE_CONST to x.

    Edge cases:
    - If x is negative, clamps to 0 before addition.
    """
    # ── Input Validation ───────────────────────────────────────────────
    x = max(0, x)

    # ── Core Logic ────────────────────────────────────────────────────
    return x + EXAMPLE_CONST

#EOF