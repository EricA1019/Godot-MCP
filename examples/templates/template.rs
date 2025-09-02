// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ File: template.rs                                                   ┃
// ┃ Purpose: Starter Rust module aligned with project commenting style  ┃
// ┃ Author: EricA1019                                                   ┃
// ┃ Last Updated: 2025-09-02                                           ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

//! Module summary: behavior and edge cases.

// ── Imports ────────────────────────────────────────────────────────────
use anyhow::Result;

// ── Constants ──────────────────────────────────────────────────────────
const EXAMPLE_CONST: u32 = 42; // inline explanation

// ── Public API ────────────────────────────────────────────────────────
/// Adds EXAMPLE_CONST to the input.
/// - Returns: sum
/// - Edge cases: saturates at u32::MAX
pub fn add_const(input: u32) -> u32 {
    input.saturating_add(EXAMPLE_CONST)
}

// ── Tests ─────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_const_basic() {
        assert_eq!(add_const(1), 43);
    }
}

//EOF