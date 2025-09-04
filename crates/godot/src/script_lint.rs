use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFinding {
    pub message: String,
    pub file: PathBuf,
}

/// Lint GDScript files under root and return findings.
pub fn lint_gd_scripts(root: &Path) -> Vec<LintFinding> {
    let re_class = Regex::new(r#"(?m)^\s*class_name\s+([A-Za-z_][A-Za-z0-9_]*)\b"#).unwrap();
    let re_debug = Regex::new(r#"(?m)^\s*(print|prints|printt)\s*\("#).unwrap();
    let re_tabs = Regex::new(r#"(?m)^\t+"#).unwrap();
    let re_ext = Regex::new(r#"(?m)\b(preload|load)\s*\(\s*\"(res://[^\"]+)\"\s*\)"#).unwrap();

    let mut out: Vec<LintFinding> = Vec::new();

    for entry in WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !entry.file_type().is_file() { continue; }
        if path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("gd")).unwrap_or(false) {
            let rel = path.strip_prefix(root).unwrap_or(path).to_path_buf();
            let Ok(contents) = fs::read_to_string(path) else { continue };

            // class_name vs filename
            if let Some(cap) = re_class.captures(&contents) {
                let cls = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let fname = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if !cls.is_empty() && !fname.eq(cls) {
                    out.push(LintFinding { message: format!("Class name mismatch: class_name {} but file is {}.gd", cls, fname), file: rel.clone() });
                }
            }

            // debug prints
            if re_debug.is_match(&contents) {
                out.push(LintFinding { message: "Debug print found".into(), file: rel.clone() });
            }

            // tabs indentation
            if re_tabs.is_match(&contents) {
                out.push(LintFinding { message: "Tab indentation used".into(), file: rel.clone() });
            }

            // missing extends
            if !contents.lines().any(|l| l.trim_start().starts_with("extends ")) {
                out.push(LintFinding { message: "Missing extends declaration".into(), file: rel.clone() });
            }

            // load/preload missing files
            for cap in re_ext.captures_iter(&contents) {
                if let Some(p) = cap.get(2).map(|m| m.as_str()) {
                    if p.starts_with("res://") {
                        let target = root.join(&p[6..]);
                        if !target.exists() {
                            out.push(LintFinding { message: format!("GDScript {} missing file: {}", cap.get(1).unwrap().as_str(), p), file: rel.clone() });
                        }
                    }
                }
            }
        }
    }

    // Deterministic ordering
    out.sort_by(|a, b| a.message.cmp(&b.message).then(a.file.cmp(&b.file)));
    out
}
