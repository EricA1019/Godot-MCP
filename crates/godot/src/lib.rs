use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GodotProjectReport {
    pub project_path: PathBuf,
    pub engine_version: Option<String>,
    pub addons: Vec<String>,
    pub export_presets: bool,
    pub warnings: Vec<String>,
}

pub fn analyze_project(root: &Path) -> Result<GodotProjectReport> {
    let mut report = GodotProjectReport::default();
    report.project_path = root.to_path_buf();

    // Detect engine version from project.godot
    let proj = root.join("project.godot");
    if let Ok(s) = std::fs::read_to_string(&proj) {
        // Basic parse for config_version or _global_script_classes, etc.
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("config_version=") {
                report.engine_version = Some(v.trim().trim_matches('\'').to_string());
                break;
            }
        }
    }

    // List addons
    let addons_dir = root.join("addons");
    if addons_dir.exists() {
        for entry in WalkDir::new(&addons_dir).max_depth(1).into_iter().flatten() {
            if entry.file_type().is_dir() && entry.path() != addons_dir {
                if let Some(name) = entry.file_name().to_str() {
                    report.addons.push(name.to_string());
                }
            }
        }
    }

    // Export presets presence
    report.export_presets = root.join("export_presets.cfg").exists();

    // Heuristic warnings
    if !report.export_presets {
        report.warnings.push("Missing export_presets.cfg".into());
    }
    if report.addons.is_empty() {
        report.warnings.push("No addons detected (addons/)".into());
    }

    Ok(report)
}
