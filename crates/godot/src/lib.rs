use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GodotProjectReport {
    pub project_path: PathBuf,
    pub project_format_version: Option<i32>,
    pub addons: Vec<String>,
    pub export_presets: Vec<ExportPreset>,
    pub issues: Vec<Issue>,
}

pub fn analyze_project(root: &Path) -> Result<GodotProjectReport> {
    let mut report = GodotProjectReport::default();
    report.project_path = root.to_path_buf();

    // Detect engine version from project.godot
    let proj = root.join("project.godot");
    if let Ok(s) = fs::read_to_string(&proj) {
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("config_version=") {
                let v = v.trim().trim_matches('\'');
                if let Ok(n) = v.parse::<i32>() { report.project_format_version = Some(n); }
            }
        }
    } else {
        report.issues.push(Issue::warn("Missing project.godot", Some(proj.strip_prefix(root).unwrap_or(&proj).to_path_buf())));
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
    } else {
        report.issues.push(Issue::info("No addons/ directory found", None));
    }

    // Export presets presence and parse
    let presets_path = root.join("export_presets.cfg");
    if presets_path.exists() {
        report.export_presets = parse_export_presets(&presets_path).unwrap_or_default();
        if report.export_presets.is_empty() {
            report.issues.push(Issue::warn("export_presets.cfg present but no presets found", Some(presets_path.strip_prefix(root).unwrap_or(&presets_path).to_path_buf())));
        }
    } else {
        report.issues.push(Issue::info("Missing export_presets.cfg", Some(presets_path.strip_prefix(root).unwrap_or(&presets_path).to_path_buf())));
    }

    // Scan .tscn and .tres for broken ext_resource paths
    report.issues.extend(scan_broken_ext_resources(root)?);

    // Deterministic ordering for stable JSON
    report.addons.sort();
    report.export_presets.sort_by(|a, b| a.name.cmp(&b.name).then(a.platform.cmp(&b.platform)));
    report.issues.sort_by(|a, b| a.severity.cmp(&b.severity).then(a.message.cmp(&b.message)));

    Ok(report)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExportPreset { pub name: String, pub platform: String }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Info, Warn, Error }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Issue { pub severity: Severity, pub message: String, pub file: Option<PathBuf> }

impl Issue {
    pub fn info<M: Into<String>>(msg: M, file: Option<PathBuf>) -> Self { Self { severity: Severity::Info, message: msg.into(), file } }
    pub fn warn<M: Into<String>>(msg: M, file: Option<PathBuf>) -> Self { Self { severity: Severity::Warn, message: msg.into(), file } }
    pub fn error<M: Into<String>>(msg: M, file: Option<PathBuf>) -> Self { Self { severity: Severity::Error, message: msg.into(), file } }
}

fn parse_export_presets(path: &Path) -> Result<Vec<ExportPreset>> {
    let s = fs::read_to_string(path)?;
    let mut out = Vec::new();
    let mut cur_name: Option<String> = None;
    let mut cur_platform: Option<String> = None;
    for line in s.lines() {
        let line = line.trim();
        if line.starts_with('[') { // new section
            if let (Some(n), Some(p)) = (cur_name.take(), cur_platform.take()) {
                out.push(ExportPreset { name: n, platform: p });
            }
            continue;
        }
        if let Some(v) = line.strip_prefix("name=") { cur_name = Some(trim_value(v)); }
        if let Some(v) = line.strip_prefix("platform=") { cur_platform = Some(trim_value(v)); }
    }
    if let (Some(n), Some(p)) = (cur_name.take(), cur_platform.take()) { out.push(ExportPreset { name: n, platform: p }); }
    Ok(out)
}

fn trim_value(v: &str) -> String { v.trim().trim_matches('\'').to_string() }

fn scan_broken_ext_resources(root: &Path) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    let re = Regex::new(r#"^\[ext_resource\s+[^\]]*path=\"([^\"]+)\""#).unwrap();
    for entry in WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() { continue; }
        let is_scene = matches!(path.extension().and_then(|s| s.to_str()), Some("tscn" | "tres"));
        if !is_scene { continue; }
        let Ok(content) = fs::read_to_string(path) else { continue };
        for line in content.lines() {
            if let Some(caps) = re.captures(line) {
                let p = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                if p.starts_with("uid://") { continue; }
                if p.starts_with("res://") {
                    let rel = &p[6..];
                    let target = root.join(rel);
                    if !target.exists() {
                        out.push(Issue::error(format!("Missing ext_resource path: {}", p), Some(path.strip_prefix(root).unwrap_or(path).to_path_buf())));
                    }
                }
            }
        }
    }
    Ok(out)
}
