// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ Crate: context                                                      ┃
// ┃ Purpose: Bundle relevant docs/code for a query using Master Index   ┃
// ┃ Author: EricA1019                                                   ┃
// ┃ Last Updated: 2025-09-02                                            ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use index::{SearchIndex, IndexPaths};

/// Max bundle size in bytes; default for v1.
pub const DEFAULT_BUNDLE_CAP: usize = 64 * 1024; // 64KB

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct BundleItem {
    pub path: String,
    pub kind: String,
    pub score: i32, // quantized score for stable ordering
    pub content: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Bundle {
    pub query: String,
    pub items: Vec<BundleItem>,
    pub size_bytes: usize,
}

fn quantize_score(score: f32) -> i32 {
    // Deterministic ordering with coarse quantization
    (score * 1000.0).round() as i32
}

// Note: helper removed to avoid dead_code warning; size is computed inline in bundle_query.

/// Create a bundle for a query using the provided index.
/// Strategy v1:
/// - Query top N (limit)
/// - Sort by quantized score desc, tie-break path asc for determinism
/// - Cap total size to DEFAULT_BUNDLE_CAP (or provided cap)
pub fn bundle_query(
    idx: &SearchIndex,
    query: &str,
    limit: usize,
    cap_bytes: Option<usize>,
) -> Result<Bundle> {
    let cap = cap_bytes.unwrap_or(DEFAULT_BUNDLE_CAP);
    let hits = idx.query_filtered(query, None, limit, true)?;

    // Map to items, keep snippet as content for brevity
    let mut items: Vec<BundleItem> = hits
        .into_iter()
        .map(|(score, path, kind, snippet)| BundleItem {
            path,
            kind,
            score: quantize_score(score),
            content: snippet.unwrap_or_default(),
        })
        .collect();

    // Sort by score desc then path asc
    items.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));

    // Enforce size cap
    let mut acc: Vec<BundleItem> = Vec::new();
    let mut total = 0usize;
    for mut it in items.into_iter() {
        // Truncate content if single item exceeds cap
        if it.content.len() > cap {
            it.content.truncate(cap);
        }
        let next = total + it.content.len() + it.path.len() + it.kind.len() + 32;
        if next > cap {
            break;
        }
        total = next;
        acc.push(it);
    }

    Ok(Bundle { query: query.to_string(), items: acc, size_bytes: total })
}

/// Convenience: open a temporary index over a root path and bundle a query.
pub fn bundle_from_root(root: &Path, data_dir: &Path, query: &str, limit: usize, cap_bytes: Option<usize>) -> Result<Bundle> {
    let paths = IndexPaths { root: root.to_path_buf(), data_dir: data_dir.to_path_buf() };
    let mut idx = SearchIndex::open(&paths)?;
    let _ = idx.scan_and_index(root)?;
    let bundle = bundle_query(&idx, query, limit, cap_bytes)?;
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn bundler_caps_and_sorts() -> Result<()> {
        let tmp = tempdir()?;
        let root = tmp.path().join("root");
        let data = tmp.path().join("data");
        fs::create_dir_all(&root)?;

        // Create a few files
        fs::write(root.join("a.txt"), "apple banana cherry")?;
        fs::write(root.join("b.md"), "banana cherry date egg")?;
        fs::write(root.join("code.rs"), "fn main() { println!(\"banana\"); } cherry")?;

        let bundle = bundle_from_root(&root, &data, "banana cherry", 10, Some(200))?;
        // Size under cap
        assert!(bundle.size_bytes <= 200);
        // Non-empty items
        assert!(!bundle.items.is_empty());
        // Deterministic order by score then path
        let mut sorted = bundle.items.clone();
        sorted.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));
        assert_eq!(bundle.items, sorted);
        Ok(())
    }
}

//EOF
