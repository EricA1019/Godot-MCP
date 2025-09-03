use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveOp {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanV1 {
    pub version: u32,
    pub operations: Vec<MoveOp>,
}

impl PlanV1 {
    pub fn new(operations: Vec<MoveOp>) -> Self {
        Self { version: 1, operations }
    }
}

/// Propose a deterministic structure normalization plan (dry-run).
/// Rules (conservative):
/// - Root-level .gd -> scripts/
/// - Root-level .tscn -> scenes/
/// - Root-level images (.png/.jpg/.jpeg) -> images/
/// Skips common folders: addons/, scripts/, scenes/, images/, examples/, docs/, target/, .git/, .vscode/
pub fn propose_plan(root: &Path) -> Result<PlanV1> {
    let root = root.canonicalize().with_context(|| format!("canonicalize root {}", root.display()))?;
    let skip_dirs = [
        "addons", "scripts", "scenes", "images", "examples", "docs", "target", ".git", ".vscode",
    ];
    let img_re: Regex = Regex::new(r"(?i)\.(png|jpg|jpeg)$").unwrap();

    let mut ops: Vec<MoveOp> = Vec::new();

    for entry in WalkDir::new(&root).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path == root { continue; }
        let name = match path.file_name().and_then(|s| s.to_str()) { Some(n) => n, None => continue };
        if entry.file_type().is_dir() {
            if skip_dirs.iter().any(|d| name == *d) { continue; }
            // Only operate on files in root; skip directories (non-recursive for safety in v1)
            continue;
        }

        // Only handle files in root directory
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let rel = path.strip_prefix(&root).unwrap().to_path_buf();
        let dest: Option<PathBuf> = match ext.as_str() {
            "gd" => Some(PathBuf::from("scripts").join(name)),
            "tscn" => Some(PathBuf::from("scenes").join(name)),
            _ => {
                if img_re.is_match(name) { Some(PathBuf::from("images").join(name)) } else { None }
            }
        };

        if let Some(dest_rel) = dest {
            // Skip if already in desired place (shouldn't be since we only look at root level files)
            if rel == dest_rel { continue; }
            ops.push(MoveOp { from: rel, to: dest_rel });
        }
    }

    // Deterministic ordering
    ops.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

    Ok(PlanV1::new(ops))
}

/// Apply a plan to the filesystem under `root`, creating parent directories as needed.
/// Returns a rollback plan (reverse operations) that can be serialized to JSON.
pub fn apply_plan(root: &Path, plan: &PlanV1) -> Result<PlanV1> {
    let root = root.canonicalize().with_context(|| format!("canonicalize root {}", root.display()))?;
    let mut reverse_ops: Vec<MoveOp> = Vec::new();
    for op in &plan.operations {
        let from_abs = root.join(&op.from);
        let to_abs = root.join(&op.to);
        if !from_abs.exists() {
            // If source doesn't exist, skip but keep reversibility pragmatic: no reverse op
            continue;
        }
        if let Some(parent) = to_abs.parent() { fs::create_dir_all(parent).with_context(|| format!("create dir {}", parent.display()))?; }
        if to_abs.exists() {
            return Err(anyhow!("destination exists: {}", to_abs.display()));
        }
        fs::rename(&from_abs, &to_abs).with_context(|| format!("move {} -> {}", from_abs.display(), to_abs.display()))?;
        reverse_ops.push(MoveOp { from: op.to.clone(), to: op.from.clone() });
    }
    // Reverse ops should be undone in reverse order to avoid conflicts
    reverse_ops.reverse();
    Ok(PlanV1::new(reverse_ops))
}

/// Roll back a previously produced rollback plan.
pub fn rollback_plan(root: &Path, rollback: &PlanV1) -> Result<()> {
    let root = root.canonicalize().with_context(|| format!("canonicalize root {}", root.display()))?;
    for op in &rollback.operations {
        let from_abs = root.join(&op.from);
        let to_abs = root.join(&op.to);
        if !from_abs.exists() { continue; }
        if let Some(parent) = to_abs.parent() { fs::create_dir_all(parent).with_context(|| format!("create dir {}", parent.display()))?; }
        if to_abs.exists() {
            return Err(anyhow!("destination exists during rollback: {}", to_abs.display()));
        }
        fs::rename(&from_abs, &to_abs).with_context(|| format!("rollback move {} -> {}", from_abs.display(), to_abs.display()))?;
    }
    Ok(())
}
