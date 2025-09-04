use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FixPlan {
    pub rules: Vec<String>,
    pub moves: Vec<FileMove>,
    pub renames: Vec<FileRename>,
    pub edits: Vec<FileEdit>,
    pub skipped: Vec<String>,
    pub stats: PlanStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PlanStats { pub scanned: usize, pub proposed: usize }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileMove { pub from: PathBuf, pub to: PathBuf }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileRename { pub from: PathBuf, pub to: PathBuf }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileEdit { pub file: PathBuf, pub kind: String, pub count: usize }

/// Build a dry-run structure fix plan. Only proposes moves for now; no apply.
/// Rules v1:
/// - .gd -> res://scripts/<filename>
/// - .tscn -> res://scenes/<filename>
/// - common assets (images/audio/fonts) -> res://assets/<relpath> (prefix with assets/ if not already)
/// Skips: addons/, crates/, docs/, target/, .git/, .import files, uid://
pub fn plan_structure_fix(root: &Path) -> FixPlan {
    let mut plan = FixPlan::default();
    plan.rules = vec![
        ".gd => res://scripts/<filename>".into(),
        ".tscn => res://scenes/<filename>".into(),
        "assets(ext) => res://assets/<relpath> (prefix)".into(),
    ];

    let asset_exts = [
        // images
        "png","jpg","jpeg","webp","svg","tga","bmp",
        // audio
        "ogg","wav","mp3",
        // fonts
        "ttf","otf",
        // shader/material/text
        "gdshader","tres",
    ];

    for entry in WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() { continue; }
        let rel = match path.strip_prefix(root) { Ok(p) => p, Err(_) => continue };

        // Skip known folders that are not part of the Godot asset tree
        if rel.starts_with("addons") || rel.starts_with("crates") || rel.starts_with("docs") || rel.starts_with("target") || rel.starts_with(".git") { continue; }
        // Skip import sidecars and lockfiles
        if rel.extension().and_then(|s| s.to_str()) == Some("import") { continue; }

        plan.stats.scanned += 1;

        let ext = rel.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        let rel_s = rel.to_string_lossy().replace('\\', "/");
        let res_from = PathBuf::from(format!("res://{}", rel_s));

        // .gd -> scripts/<filename>
        if ext == "gd" {
            if rel.components().next().map(|c| c.as_os_str()) == Some(std::ffi::OsStr::new("scripts")) {
                continue; // already under scripts
            }
            let fname = rel.file_name().unwrap().to_string_lossy().to_string();
            let to = PathBuf::from(format!("res://scripts/{}", fname));
            if to != res_from {
                plan.moves.push(FileMove { from: res_from, to });
            }
            continue;
        }

        // .tscn -> scenes/<filename>
        if ext == "tscn" {
            if rel.components().next().map(|c| c.as_os_str()) == Some(std::ffi::OsStr::new("scenes")) {
                continue; // already under scenes
            }
            let fname = rel.file_name().unwrap().to_string_lossy().to_string();
            let to = PathBuf::from(format!("res://scenes/{}", fname));
            if to != res_from {
                plan.moves.push(FileMove { from: res_from, to });
            }
            continue;
        }

        // asset ext -> assets/<relpath> (prefix)
        if asset_exts.contains(&ext.as_str()) {
            if rel.components().next().map(|c| c.as_os_str()) == Some(std::ffi::OsStr::new("assets")) {
                continue; // already under assets
            }
            let to = PathBuf::from(format!("res://assets/{}", rel_s));
            if to != res_from {
                plan.moves.push(FileMove { from: res_from, to });
            }
            continue;
        }
    }

    plan.moves.sort();
    plan.stats.proposed = plan.moves.len();
    plan
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ApplySummary {
    pub moved: Vec<FileMove>,
    pub edited: Vec<FileEdit>,
    pub backed_up: usize,
}

/// Apply a previously generated plan: move files and update references in .tscn/.tres/.gd.
/// Creates backups under .structure_fix/backup before moving.
pub fn apply_structure_fix(root: &Path, plan: &FixPlan) -> Result<ApplySummary> {
    // Build mapping of res://old -> res://new
    let mut mapping: Vec<(String, String)> = Vec::new();
    for mv in &plan.moves {
        let old = mv.from.to_string_lossy().to_string();
        let newp = mv.to.to_string_lossy().to_string();
        if old == newp { continue; }
        mapping.push((old, newp));
    }
    // Move files with backup
    let mut summary = ApplySummary::default();
    let backup_root = root.join(".structure_fix/backup");
    for mv in &plan.moves {
        let from_res = mv.from.to_string_lossy().to_string();
        let to_res = mv.to.to_string_lossy().to_string();
        if from_res == to_res { continue; }

        let from_fs = res_to_fs(root, &from_res)?;
        let to_fs = res_to_fs(root, &to_res)?;
        if !from_fs.exists() {
            // If already moved, skip
            continue;
        }
        // Backup original
        let backup_path = backup_root.join(from_fs.strip_prefix(root).unwrap_or(&from_fs));
        if let Some(parent) = backup_path.parent() { fs::create_dir_all(parent)?; }
        fs::copy(&from_fs, &backup_path)?;
        summary.backed_up += 1;
        // Ensure target dir exists
        if let Some(parent) = to_fs.parent() { fs::create_dir_all(parent)?; }
        // Perform move (rename)
        fs::rename(&from_fs, &to_fs)?;
        summary.moved.push(FileMove { from: mv.from.clone(), to: mv.to.clone() });
    }

    // Update references
    let exts_requiring_extres_scan = ["tscn", "tres"];
    let re_ext = Regex::new(r#"^\[ext_resource\s+[^\]]*path=\"([^\"]+)\""#).unwrap();
    let re_gd = Regex::new(r#"(?m)\b(preload|load)\s*\(\s*\"(res://[^\"]+)\"\s*\)"#).unwrap();

    for entry in WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() { continue; }
        let rel = match path.strip_prefix(root) { Ok(p) => p, Err(_) => continue };
        // Skip backups and non-project dirs
        if rel.starts_with(".structure_fix") || rel.starts_with("target") || rel.starts_with(".git") || rel.starts_with("docs") || rel.starts_with("crates") { continue; }

        let ext = rel.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        let is_tscn_or_tres = exts_requiring_extres_scan.contains(&ext.as_str());
        let is_gd = ext == "gd";
        if !is_tscn_or_tres && !is_gd { continue; }

        let Ok(contents) = fs::read_to_string(path) else { continue };
        let mut edits = 0usize;
        let mut newc = String::new();
        if is_tscn_or_tres {
            for line in contents.lines() {
                if let Some(cap) = re_ext.captures(line) {
                    let p = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    if let Some((_, newp)) = mapping.iter().find(|(old, _)| old == p) {
                        let replaced = line.replacen(p, newp, 1);
                        newc.push_str(&replaced);
                        newc.push('\n');
                        edits += 1;
                        continue;
                    }
                }
                newc.push_str(line);
                newc.push('\n');
            }
        } else {
            // .gd: update preload/load occurrences
            let mut last = 0usize;
            for m in re_gd.captures_iter(&contents) {
                let m0 = m.get(0).unwrap();
                newc.push_str(&contents[last..m0.start()]);
                let whole = m0.as_str();
                let p = m.get(2).map(|mm| mm.as_str()).unwrap_or("");
                if let Some((_, newp)) = mapping.iter().find(|(old, _)| old == p) {
                    let replaced = whole.replacen(p, newp, 1);
                    newc.push_str(&replaced);
                    edits += 1;
                } else {
                    newc.push_str(whole);
                }
                last = m0.end();
            }
            newc.push_str(&contents[last..]);
        }

        if edits > 0 {
            fs::write(path, newc)?;
            summary.edited.push(FileEdit { file: rel.to_path_buf(), kind: if is_gd { "gd-load-preload".into() } else { "ext_resource-path".into() }, count: edits });
        }
    }

    Ok(summary)
}

fn res_to_fs(root: &Path, res_uri: &str) -> Result<PathBuf> {
    if !res_uri.starts_with("res://") {
        return Err(anyhow!("not a res:// uri: {}", res_uri));
    }
    Ok(root.join(&res_uri[6..]))
}
