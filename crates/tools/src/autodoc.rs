use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::{Path, PathBuf}};

/// Contract
/// Inputs: workspace root path
/// Outputs: created-or-verified files list
/// Error modes: IO errors on read/write; permissions; invalid templates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoDocReport {
    pub created: Vec<PathBuf>,
    pub verified: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub updated: Vec<PathBuf>,
}

impl AutoDocReport {
    pub fn empty() -> Self { Self { created: vec![], verified: vec![], skipped: vec![], updated: vec![] } }
}

/// Minimal CTS templates we ensure exist.
fn targets() -> Vec<(PathBuf, &'static str)> {
    vec![
        (PathBuf::from("docs/DEV_LOG.md"), DEV_LOG_TEMPLATE),
        (PathBuf::from("docs/PROJECT_INDEX.md"), PROJECT_INDEX_TEMPLATE),
        (PathBuf::from("docs/WORKFLOW_PROJECT.md"), WORKFLOW_PROJECT_TEMPLATE),
    (PathBuf::from("CHANGELOG.md"), CHANGELOG_TEMPLATE),
    (PathBuf::from("CONTRIBUTING.md"), CONTRIBUTING_TEMPLATE),
    ]
}

/// Ensure docs exist and managed regions are present/updated.
/// If dry_run = true, report what would change without writing.
pub fn ensure_autodocs(root: &Path) -> Result<AutoDocReport> { ensure_autodocs_opts(root, EnsureOpts::default()) }

#[derive(Debug, Clone, Copy, Default)]
pub struct EnsureOpts {
    pub dry_run: bool,
    pub check_only: bool,
}

pub fn ensure_autodocs_opts(root: &Path, opts: EnsureOpts) -> Result<AutoDocReport> {
    let mut report = AutoDocReport::empty();

    for (rel, template) in targets() {
        let path = root.join(&rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let desired = merge_with_region(&content, template);
            if normalize_newlines(&content) != normalize_newlines(&desired) {
                if opts.dry_run || opts.check_only {
                    report.updated.push(rel);
                } else {
                    atomic_write(&path, desired.as_bytes())?;
                    report.updated.push(rel);
                }
            } else {
                report.verified.push(rel);
            }
        } else {
            if opts.dry_run || opts.check_only {
                report.created.push(rel);
            } else {
                std::fs::create_dir_all(path.parent().unwrap())?;
                atomic_write(&path, template.as_bytes())?;
                report.created.push(rel);
            }
        }
    }

    Ok(report)
}

const BEGIN: &str = "<!-- AUTODOC:BEGIN main -->";
const END: &str = "<!-- AUTODOC:END main -->";

fn merge_with_region(existing: &str, template: &str) -> String {
    // If existing has region markers, only replace the region; otherwise, append a managed region block non-destructively.
    if let (Some(b), Some(e)) = (existing.find(BEGIN), existing.find(END)) {
        let prefix = &existing[..b + BEGIN.len()];
        let suffix = &existing[e..];
        let (tb, te) = (template.find(BEGIN), template.find(END));
        let region = if let (Some(tb), Some(te)) = (tb, te) {
            &template[tb + BEGIN.len()..te]
        } else {
            template
        };
        format!("{prefix}{region}{suffix}")
    } else {
        // Append the region block from template (including markers) to preserve existing content.
        if let (Some(tb), Some(te)) = (template.find(BEGIN), template.find(END)) {
            let region_block = &template[tb..te + END.len()];
            let mut out = String::new();
            out.push_str(existing);
            if !existing.ends_with('\n') { out.push('\n'); }
            out.push('\n');
            out.push_str(region_block);
            out.push('\n');
            out
        } else {
            // No markers in template; be conservative and append the whole template with spacing.
            let mut out = String::new();
            out.push_str(existing);
            if !existing.ends_with('\n') { out.push('\n'); }
            out.push('\n');
            out.push_str(template);
            if !template.ends_with('\n') { out.push('\n'); }
            out
        }
    }
}

fn normalize_newlines(s: &str) -> String { s.replace("\r\n", "\n") }

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("no parent for path")?;
    std::fs::create_dir_all(parent)?;
    let mut tmp = parent.to_path_buf();
    tmp.push(format!(".{}.__autodoc_tmp", path.file_name().unwrap().to_string_lossy()));
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

const DEV_LOG_TEMPLATE: &str = r#"# Dev Log

<!-- AUTODOC:BEGIN main -->
- Use this log to capture hop-by-hop notes, decisions, and follow-ups.
<!-- AUTODOC:END main -->

"#;

const PROJECT_INDEX_TEMPLATE: &str = r#"# Project Index

<!-- AUTODOC:BEGIN main -->
- Inventory of code, tools, docs.
<!-- AUTODOC:END main -->

"#;

const WORKFLOW_PROJECT_TEMPLATE: &str = r#"# Project Workflow

<!-- AUTODOC:BEGIN main -->
- Close-to-Shore: tiny hops, green builds, clear acceptance.
<!-- AUTODOC:END main -->

"#;

const CHANGELOG_TEMPLATE: &str = r#"# Changelog

All notable changes to this project will be documented in this file.

<!-- AUTODOC:BEGIN main -->
## [Unreleased]
- Start listing your changes here following Keep a Changelog style.
<!-- AUTODOC:END main -->

"#;

const CONTRIBUTING_TEMPLATE: &str = r#"# Contributing

Thanks for your interest in contributing!

<!-- AUTODOC:BEGIN main -->
- Use small, reviewable PRs. Keep builds green and add tests when changing behavior.
- Follow the CTS workflow: tiny hops, deterministic outcomes, and clear acceptance.
- Run `cargo test` and `cargo clippy -D warnings` locally before pushing.
<!-- AUTODOC:END main -->

"#;
