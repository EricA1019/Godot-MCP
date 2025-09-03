use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};
use tracing::info;

/// Contract
/// Inputs: workspace root path
/// Outputs: created-or-verified files list
/// Error modes: IO errors on read/write; permissions; invalid templates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoDocReport {
    pub created: Vec<PathBuf>,
    pub verified: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
}

impl AutoDocReport {
    pub fn empty() -> Self { Self { created: vec![], verified: vec![], skipped: vec![] } }
}

/// Minimal CTS templates we ensure exist.
fn targets() -> Vec<(PathBuf, &'static str)> {
    vec![
        (PathBuf::from("docs/DEV_LOG.md"), DEV_LOG_TEMPLATE),
        (PathBuf::from("docs/PROJECT_INDEX.md"), PROJECT_INDEX_TEMPLATE),
        (PathBuf::from("docs/WORKFLOW_PROJECT.md"), WORKFLOW_PROJECT_TEMPLATE),
    ]
}

pub fn ensure_autodocs(root: &Path) -> Result<AutoDocReport> {
    let mut report = AutoDocReport::empty();

    for (rel, template) in targets() {
        let path = root.join(&rel);
        if path.exists() {
            // lightweight verification: file non-empty and contains marker
            let content = fs::read_to_string(&path).unwrap_or_default();
            if content.trim().is_empty() || !content.contains("#") {
                info!(file=%path.display(), "doc existed but looked empty; refreshing with template");
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(&path, template)?;
                report.created.push(rel);
            } else {
                report.verified.push(rel);
            }
        } else {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, template)?;
            report.created.push(rel);
        }
    }

    Ok(report)
}

const DEV_LOG_TEMPLATE: &str = r#"# Dev Log

- Use this log to capture hop-by-hop notes, decisions, and follow-ups.

"#;

const PROJECT_INDEX_TEMPLATE: &str = r#"# Project Index

- Inventory of code, tools, docs.

"#;

const WORKFLOW_PROJECT_TEMPLATE: &str = r#"# Project Workflow

- Close-to-Shore: tiny hops, green builds, clear acceptance.

"#;
