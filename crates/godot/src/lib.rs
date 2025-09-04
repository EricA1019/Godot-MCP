use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub mod scene_validate;
pub mod signal_validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GodotProjectReport {
    pub project_path: PathBuf,
    pub project_format_version: Option<i32>,
    pub addons: Vec<String>,
    pub export_presets: Vec<ExportPreset>,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneCheckOptions {
    pub script: bool,
    pub properties: bool,
    pub subresource: bool,
    pub preload: bool,
    pub load: bool,
}

impl Default for SceneCheckOptions {
    fn default() -> Self {
        Self { script: true, properties: true, subresource: true, preload: true, load: true }
    }
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
        // Application icon and main scene checks (heuristic INI parsing)
        let icon = find_ini_kv(&s, "config/icon");
        if let Some(v) = icon {
            if let Some(p) = v.strip_prefix("res://") {
                let t = root.join(p);
                if !t.exists() { report.issues.push(Issue::warn(format!("Missing application icon: {}", v), Some(proj.strip_prefix(root).unwrap_or(&proj).to_path_buf()))); }
            }
        } else {
            report.issues.push(Issue::info("No application icon configured (config/icon)", Some(proj.strip_prefix(root).unwrap_or(&proj).to_path_buf())));
        }
        let main_scene = find_ini_kv(&s, "run/main_scene");
        if let Some(v) = main_scene {
            if let Some(p) = v.strip_prefix("res://") {
                let t = root.join(p);
                if !t.exists() { report.issues.push(Issue::warn(format!("Missing main scene: {}", v), Some(proj.strip_prefix(root).unwrap_or(&proj).to_path_buf()))); }
            }
        } else {
            report.issues.push(Issue::info("No main scene configured (run/main_scene)", Some(proj.strip_prefix(root).unwrap_or(&proj).to_path_buf())));
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
                    // addon health: plugin.cfg presence
                    let plugin_cfg = entry.path().join("plugin.cfg");
                    if !plugin_cfg.exists() {
                        report.issues.push(Issue::warn(format!("Addon '{}' missing plugin.cfg", name), Some(plugin_cfg.strip_prefix(root).unwrap_or(&plugin_cfg).to_path_buf())));
                    }
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
        // Validate export_path parent directories exist (heuristic)
        for p in &report.export_presets {
            if let Some(path) = &p.export_path {
                let joined = if Path::new(path).is_absolute() { PathBuf::from(path) } else { root.join(path) };
                if let Some(parent) = joined.parent() {
                    if !parent.exists() {
                        report.issues.push(Issue::info(format!("Export path parent directory does not exist: {}", parent.display()), Some(presets_path.strip_prefix(root).unwrap_or(&presets_path).to_path_buf())));
                    }
                }
            }
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

/// Run signal validation across .tscn files and convert to Issue entries.
pub fn signal_issues_as_report(root: &Path) -> Vec<Issue> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() { continue; }
        let is_scene = matches!(path.extension().and_then(|s| s.to_str()), Some("tscn"));
        if !is_scene { continue; }
        let rel = path.strip_prefix(root).unwrap_or(path);
        let sig_issues = signal_validate::validate_scene_signals(root, rel);
        for si in sig_issues {
            let mut msg = si.message.clone();
            if let Some(np) = si.node_path.as_ref() {
                msg = format!("{} [node: {}]", msg, np);
            }
            out.push(Issue::error(msg, Some(rel.to_path_buf())));
        }
    }
    out
}

/// Build a DOT graph of signal connections across all .tscn files under root.
pub fn signal_graph_dot(root: &Path) -> String {
    let mut edges: Vec<signal_validate::ConnectionEdge> = Vec::new();
    for entry in WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() { continue; }
        if matches!(path.extension().and_then(|s| s.to_str()), Some("tscn")) {
            let rel = path.strip_prefix(root).unwrap_or(path);
            edges.extend(signal_validate::extract_scene_connections(root, rel));
        }
    }
    // Sort once more globally to ensure deterministic output across files
    edges.sort();
    signal_validate::connections_to_dot(&edges)
}

/// Run scene validation across .tscn files and convert to Issue entries.
/// Skips generic ext_resource path issues to avoid duplication with scan_broken_ext_resources.
pub fn scene_issues_as_report(root: &Path) -> Vec<Issue> {
    scene_issues_as_report_with(root, &SceneCheckOptions::default())
}

/// Same as scene_issues_as_report but allows selecting which checks to keep.
pub fn scene_issues_as_report_with(root: &Path, opts: &SceneCheckOptions) -> Vec<Issue> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() { continue; }
        let is_scene = matches!(path.extension().and_then(|s| s.to_str()), Some("tscn"));
        if !is_scene { continue; }
        let rel = path.strip_prefix(root).unwrap_or(path);
        let scene_issues = scene_validate::validate_scene(root, rel);
        for si in scene_issues {
            // Avoid duplicating the broad ext_resource missing messages already emitted by scan_broken_ext_resources
            if si.message.starts_with("Missing ext_resource path:") { continue; }

            // Filter by selected checks
            match scene_issue_kind(&si.message) {
                SceneIssueKind::MissingScript | SceneIssueKind::ScriptExtResourceMissing | SceneIssueKind::UnknownExtResource => {
                    if !opts.script { continue; }
                }
                SceneIssueKind::PropertyExtMissing => {
                    if !opts.properties { continue; }
                }
                SceneIssueKind::UnknownSubResource => {
                    if !opts.subresource { continue; }
                }
                SceneIssueKind::PreloadMissing => {
                    if !opts.preload { continue; }
                }
                SceneIssueKind::LoadMissing => {
                    if !opts.load { continue; }
                }
                SceneIssueKind::Other => {}
            }
            let mut msg = si.message.clone();
            if let Some(np) = si.node_path.as_ref() {
                msg = format!("{} [node: {}]", msg, np);
            }
            // Map all scene validator findings to Error for now
            out.push(Issue::error(msg, Some(rel.to_path_buf())));
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SceneIssueKind { MissingScript, ScriptExtResourceMissing, UnknownExtResource, PropertyExtMissing, UnknownSubResource, PreloadMissing, LoadMissing, Other }

fn scene_issue_kind(msg: &str) -> SceneIssueKind {
    if msg.starts_with("Missing script:") { return SceneIssueKind::MissingScript; }
    if msg.starts_with("Script ExtResource(") { return SceneIssueKind::ScriptExtResourceMissing; }
    if msg.starts_with("Unknown ExtResource id:") { return SceneIssueKind::UnknownExtResource; }
    if msg.starts_with("Property '") { return SceneIssueKind::PropertyExtMissing; }
    if msg.starts_with("Unknown SubResource id:") { return SceneIssueKind::UnknownSubResource; }
    if msg.starts_with("Preload missing file:") { return SceneIssueKind::PreloadMissing; }
    if msg.starts_with("Load missing file:") { return SceneIssueKind::LoadMissing; }
    SceneIssueKind::Other
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExportPreset { pub name: String, pub platform: String, pub export_path: Option<String> }

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
    let mut cur_export_path: Option<String> = None;
    for line in s.lines() {
        let line = line.trim();
        if line.starts_with('[') { // new section
            if let (Some(n), Some(p)) = (cur_name.take(), cur_platform.take()) {
                out.push(ExportPreset { name: n, platform: p, export_path: cur_export_path.take() });
            }
            continue;
        }
        if let Some(v) = line.strip_prefix("name=") { cur_name = Some(trim_value(v)); }
        if let Some(v) = line.strip_prefix("platform=") { cur_platform = Some(trim_value(v)); }
        if let Some(v) = line.strip_prefix("export_path=") { cur_export_path = Some(trim_value(v)); }
    }
    if let (Some(n), Some(p)) = (cur_name.take(), cur_platform.take()) { out.push(ExportPreset { name: n, platform: p, export_path: cur_export_path.take() }); }
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

fn find_ini_kv(contents: &str, key: &str) -> Option<String> {
    // Search for lines like key="res://..." possibly with section headers above
    for line in contents.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix(&format!("{key}=")) { return Some(trim_value(v)); }
    }
    None
}

// --- Outputs ---
pub fn to_sarif(report: &GodotProjectReport) -> serde_json::Value {
    let results: Vec<serde_json::Value> = report.issues.iter().map(|i| {
        let level = match i.severity { Severity::Info => "note", Severity::Warn => "warning", Severity::Error => "error" };
        let rule_id = classify_rule_id(i);
        serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {"text": i.message},
            "locations": [{ "physicalLocation": { "artifactLocation": { "uri": i.file.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default() } } }]
        })
    }).collect();
    serde_json::json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {"driver": {
                "name": "godot-analyzer",
                "rules": [
                    {"id": "godot-analyzer", "name": "godot-analyzer", "shortDescription": {"text": "Godot project configuration checks"}},
                    {"id": "scene-validator", "name": "scene-validator", "shortDescription": {"text": "Godot scene (.tscn) validation checks"}},
                    {"id": "signal-validator", "name": "signal-validator", "shortDescription": {"text": "Godot scene signal connection checks"}}
                ]
            }},
            "results": results
        }]
    })
}

fn classify_rule_id(i: &Issue) -> &'static str {
    // Heuristic mapping: known scene validator signatures get a distinct rule id
    let msg = i.message.as_str();
    if msg.starts_with("Missing script:")
        || msg.starts_with("Script ExtResource(")
    || msg.starts_with("Unknown ExtResource id:")
    || msg.starts_with("Property '")
    || msg.starts_with("Unknown SubResource id:")
    || msg.starts_with("Preload missing file:")
    || msg.starts_with("Load missing file:")
    {
        "scene-validator"
    } else if msg.starts_with("Unknown connection '")
        || msg.starts_with("Connection missing ")
        || msg.starts_with("Duplicate connection:")
        || msg.starts_with("Invalid method name:")
        || msg.starts_with("Target method not found:")
    {
        "signal-validator"
    } else {
        // Default to the core analyzer
        "godot-analyzer"
    }
}

pub fn to_junit(report: &GodotProjectReport) -> String {
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(&format!("<testsuite name=\"godot-analyzer\" tests=\"{}\">\n", report.issues.len()));
    for i in &report.issues {
        let name = format!("{}", i.message);
    let class_name = match classify_rule_id(i) { "scene-validator" => "scene-validator", "signal-validator" => "signal-validator", _ => "godot-analyzer" };
    s.push_str(&format!("  <testcase name=\"{}\" classname=\"{}\">\n", xml_escape(&name), class_name));
        s.push_str(&format!("    <failure message=\"{:?}\">{}</failure>\n", i.severity, xml_escape(&i.file.as_ref().map(|p| p.display().to_string()).unwrap_or_default())));
        s.push_str("  </testcase>\n");
    }
    s.push_str("</testsuite>\n");
    s
}

fn xml_escape(input: &str) -> String { input.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }
