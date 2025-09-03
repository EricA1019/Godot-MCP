use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub updated: Option<PathBuf>,
}

pub fn run(root: &Path) -> Result<Report> {
    let findings = classify(root)?;
    let updated = update_project_index(root, &findings)?;
    Ok(Report { findings, updated })
}

pub fn classify(root: &Path) -> Result<Vec<Finding>> {
    let mut out = Vec::new();
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

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
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        // Temp/edit artifacts
        if name.ends_with('~') || name == ".DS_Store" || name == "Thumbs.db" || name.ends_with(".swp") || name.ends_with(".tmp") {
            out.push(Finding { kind: "temp".into(), path: rel, reason: "Editor/OS temp artifact".into(), bytes: entry.metadata().ok().map(|m| m.len()) });
            continue;
        }

        // Orphan Godot .import (e.g., image.png.import without image.png)
        if name.ends_with(".import") {
            let stem = name.trim_end_matches(".import");
            let sibling = path.parent().unwrap_or(Path::new("")).join(stem);
            if !sibling.exists() {
                out.push(Finding { kind: "orphan_import".into(), path: rel, reason: format!("Missing source for {}", stem), bytes: entry.metadata().ok().map(|m| m.len()) });
                continue;
            }
        }

        // Large files (> 5 MiB) outside known docs content
        if let Ok(meta) = entry.metadata() {
            let len = meta.len();
            if len > 5 * 1024 * 1024 {
                if !(path.components().any(|c| c.as_os_str() == "rust-book") || path.components().any(|c| c.as_os_str() == "docs")) {
                    out.push(Finding { kind: "large".into(), path: rel, reason: "Large file (>5MiB)".into(), bytes: Some(len) });
                }
            }
        }

        // Orphan image import variants (e.g., .png.import is handled above). Detect .png.import without .png handled.
        if ext == "import" { /* already handled */ }
    }

    // Deterministic ordering
    out.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.path.cmp(&b.path)));
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
        s.push_str(&format!("- [{}] {} — {}{}\n", f.kind, f.path.display(), f.reason, size));
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
