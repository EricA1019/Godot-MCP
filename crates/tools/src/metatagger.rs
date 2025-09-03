use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const CLEANUP_BEGIN: &str = "<!-- METATAGGER:BEGIN cleanup -->";
const CLEANUP_END: &str = "<!-- METATAGGER:END cleanup -->";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    pub kind: String,
    pub path: PathBuf,
    pub reason: String,
    pub bytes: Option<u64>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Info, Warn, Error }

impl Default for Severity { fn default() -> Self { Severity::Warn } }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub updated: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct IgnoreConfig { set: Option<GlobSet> }

fn load_ignores(root: &Path) -> Result<IgnoreConfig> {
    let mut builder = GlobSetBuilder::new();
    let ignore_path = root.join(".metataggerignore");
    if ignore_path.exists() {
        let content = fs::read_to_string(&ignore_path)?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let glob = Glob::new(line).map_err(|e| anyhow!("bad ignore pattern '{line}': {e}"))?;
            builder.add(glob);
        }
        let set = builder.build().ok();
        return Ok(IgnoreConfig { set });
    }
    Ok(IgnoreConfig { set: None })
}

pub fn run(root: &Path) -> Result<Report> {
    let cfg = load_ignores(root)?;
    let findings = classify(root, &cfg)?;
    let updated = update_project_index(root, &findings)?;
    Ok(Report { findings, updated })
}

pub fn classify(root: &Path, ignores: &IgnoreConfig) -> Result<Vec<Finding>> {
    let mut out = Vec::new();
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    // First pass: collect file metadata, hashes for duplicate detection, and references
    let mut by_hash: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut image_sources: BTreeSet<PathBuf> = BTreeSet::new();
    let mut image_imports: BTreeSet<PathBuf> = BTreeSet::new();
    let mut export_presets: Option<PathBuf> = None;

    for entry in WalkDir::new(&root).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if entry.file_type().is_dir() {
            // Skip common build/vendor dirs
            let name = entry.file_name().to_string_lossy();
            if matches!(name.as_ref(), ".git" | "target" | ".idea" | ".vscode" | "node_modules") {
                continue;
            }
            if path.starts_with(root.join("target")) || path.starts_with(root.join(".git")) {
                continue;
            }
            continue;
        }

        let rel = path.strip_prefix(&root).unwrap_or(path).to_path_buf();
        if let Some(set) = &ignores.set { if set.is_match(&rel) { continue; } }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        // Collect image sources and imports
        match ext {
            "png" | "jpg" | "jpeg" | "webp" | "svg" | "gif" => { image_sources.insert(rel.clone()); },
            "import" => if name.ends_with(".png.import") || name.ends_with(".jpg.import") || name.ends_with(".jpeg.import") || name.ends_with(".webp.import") || name.ends_with(".svg.import") || name.ends_with(".gif.import") { image_imports.insert(rel.clone()); },
            _ => {}
        }

        if name == "export_presets.cfg" { export_presets = Some(rel.clone()); }

        // Temp/edit artifacts
        if name.ends_with('~') || name == ".DS_Store" || name == "Thumbs.db" || name.ends_with(".swp") || name.ends_with(".tmp") {
            out.push(Finding { kind: "temp".into(), path: rel.clone(), reason: "Editor/OS temp artifact".into(), bytes: entry.metadata().ok().map(|m| m.len()), severity: Severity::Info });
            continue;
        }

        // Orphan Godot .import (e.g., image.png.import without image.png)
        if name.ends_with(".import") {
            let stem = name.trim_end_matches(".import");
            let sibling = path.parent().unwrap_or(Path::new("")).join(stem);
            if !sibling.exists() {
                out.push(Finding { kind: "orphan_import".into(), path: rel.clone(), reason: format!("Missing source for {}", stem), bytes: entry.metadata().ok().map(|m| m.len()), severity: Severity::Warn });
                continue;
            }
        }

        // Large files (> 5 MiB) outside known docs content
        if let Ok(meta) = entry.metadata() {
            let len = meta.len();
            if len > 5 * 1024 * 1024 {
                if !(path.components().any(|c| c.as_os_str() == "rust-book") || path.components().any(|c| c.as_os_str() == "docs")) {
                    out.push(Finding { kind: "large".into(), path: rel.clone(), reason: "Large file (>5MiB)".into(), bytes: Some(len), severity: Severity::Warn });
                }
            }
        }

        // Orphan image import variants (e.g., .png.import is handled above). Detect .png.import without .png handled.
        if ext == "import" { /* already handled */ }

        // Hash for duplicate detection (limit to common binary/text assets)
        if let Ok(bytes) = fs::read(path) {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let hash = format!("{:x}", hasher.finalize());
            by_hash.entry(hash).or_default().push(rel.clone());
        }
    }

    // Unused images: .png with no matching .import (heuristic)
    for src in &image_sources {
        let import = PathBuf::from(format!("{}.import", src.display()));
        if !image_imports.contains(&import) {
            out.push(Finding { kind: "unused_image".into(), path: src.clone(), reason: "Image source has no .import (likely unused)".into(), bytes: fs::metadata(root.join(src)).ok().map(|m| m.len()), severity: Severity::Info });
        }
    }

    // Duplicated assets by content hash (two or more distinct paths with same hash)
    for (_hash, paths) in by_hash.into_iter() {
        if paths.len() >= 2 {
            for p in paths {
                out.push(Finding { kind: "duplicate".into(), path: p, reason: "Same content exists at multiple paths".into(), bytes: None, severity: Severity::Warn });
            }
        }
    }

    // Stale export presets (exists but missing default preset markers)
    if let Some(p) = export_presets {
        if let Ok(s) = fs::read_to_string(root.join(&p)) {
            if !s.contains("[preset.0]") && !s.contains("[preset]") {
                out.push(Finding { kind: "export_presets".into(), path: p, reason: "export_presets.cfg present but no presets defined".into(), bytes: None, severity: Severity::Info });
            }
        }
    }

    // Deterministic ordering
    out.sort_by(|a, b| a.severity.cmp(&b.severity).then(a.kind.cmp(&b.kind)).then(a.path.cmp(&b.path)));
    Ok(out)
}

fn update_project_index(root: &Path, findings: &[Finding]) -> Result<Option<PathBuf>> {
    let proj = root.join("docs/PROJECT_INDEX.md");
    if !proj.exists() {
        // Minimal scaffold including METATAGGER region
        let tmpl = "# Project Index\n\n<!-- AUTODOC:BEGIN main -->\n- Inventory of code, tools, docs.\n<!-- AUTODOC:END main -->\n\n## Cleanup candidates\n\n<!-- METATAGGER:BEGIN cleanup -->\nNone yet.\n<!-- METATAGGER:END cleanup -->\n\n";
        atomic_write(&proj, tmpl.as_bytes())?;
    }
    let content = fs::read_to_string(&proj).unwrap_or_default();
    let new_block = render_cleanup_block(findings);
    let updated = replace_region(&content, CLEANUP_BEGIN, CLEANUP_END, &new_block);
    if normalize_newlines(&content) != normalize_newlines(&updated) {
        atomic_write(&proj, updated.as_bytes())?;
        Ok(Some(proj))
    } else {
        Ok(None)
    }
}

fn render_cleanup_block(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "None found.".to_string();
    }
    let mut s = String::new();
    for f in findings {
        let size = f.bytes.map(|b| format!(" ({} bytes)", b)).unwrap_or_default();
    s.push_str(&format!("- [{}][{:?}] {} — {}{}\n", f.kind, f.severity, f.path.display(), f.reason, size));
    }
    s
}

fn replace_region(existing: &str, begin: &str, end: &str, body: &str) -> String {
    if let (Some(b), Some(e)) = (existing.find(begin), existing.find(end)) {
        let prefix = &existing[..b + begin.len()];
        let suffix = &existing[e..];
        format!("{prefix}\n{body}{suffix}")
    } else {
        // Append new region at end
        let mut out = existing.to_string();
        if !out.ends_with('\n') { out.push('\n'); }
        out.push_str(begin);
        out.push('\n');
        out.push_str(body);
        out.push_str(end);
        out.push('\n');
        out
    }
}

fn normalize_newlines(s: &str) -> String { s.replace("\r\n", "\n") }

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("no parent for path")?;
    fs::create_dir_all(parent)?;
    let mut tmp = parent.to_path_buf();
    tmp.push(format!(".{}.__metatagger_tmp", path.file_name().unwrap().to_string_lossy()));
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}
