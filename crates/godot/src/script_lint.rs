use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::Severity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFinding {
    pub code: String,
    pub message: String,
    pub file: PathBuf,
    pub severity: Severity,
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

            // Parse suppression directives and severity override
            // Supported:
            //   # gd-lint: off                      -> disable all rules in this file
            //   # gd-lint: disable=rule1,rule2,...  -> disable listed rules
            //   # gd-lint: level=info|warn|error     -> set severity for this file's lints
            let (disable_all, disabled, level) = parse_controls(&contents);
            let sev = level.unwrap_or(Severity::Warn);
            if disable_all { continue; }

            // class_name vs filename
            if let Some(cap) = re_class.captures(&contents) {
                let cls = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let fname = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if !cls.is_empty() && !fname.eq(cls) {
                    if !disabled.contains("class-name-mismatch") {
                        out.push(LintFinding { code: "class-name-mismatch".into(), message: format!("Class name mismatch: class_name {} but file is {}.gd", cls, fname), file: rel.clone(), severity: sev });
                    }
                }
            }

            // debug prints
            if re_debug.is_match(&contents) {
                if !disabled.contains("debug-print") {
                    out.push(LintFinding { code: "debug-print".into(), message: "Debug print found".into(), file: rel.clone(), severity: sev });
                }
            }

            // tabs indentation
            if re_tabs.is_match(&contents) {
                if !disabled.contains("tab-indentation") {
                    out.push(LintFinding { code: "tab-indentation".into(), message: "Tab indentation used".into(), file: rel.clone(), severity: sev });
                }
            }

            // missing extends
            if !contents.lines().any(|l| l.trim_start().starts_with("extends ")) {
                if !disabled.contains("missing-extends") {
                    out.push(LintFinding { code: "missing-extends".into(), message: "Missing extends declaration".into(), file: rel.clone(), severity: sev });
                }
            }

            // load/preload missing files
            for cap in re_ext.captures_iter(&contents) {
                if let Some(p) = cap.get(2).map(|m| m.as_str()) {
                    if p.starts_with("res://") {
                        let target = root.join(&p[6..]);
                        if !target.exists() {
                            if !disabled.contains("missing-resource-ref") {
                                out.push(LintFinding { code: "missing-resource-ref".into(), message: format!("GDScript {} missing file: {}", cap.get(1).unwrap().as_str(), p), file: rel.clone(), severity: sev });
                            }
                        }
                    }
                }
            }
        }
    }

    // Deterministic ordering
    out.sort_by(|a, b| a.code.cmp(&b.code).then(a.message.cmp(&b.message)).then(a.file.cmp(&b.file)));
    out
}

fn parse_controls(contents: &str) -> (bool, HashSet<String>, Option<Severity>) {
    let mut disabled: HashSet<String> = HashSet::new();
    let mut off = false;
    let mut level: Option<Severity> = None;
    for line in contents.lines() {
        let line = line.trim();
        if let Some(idx) = line.find('#') {
            let c = &line[idx + 1..].trim();
            if let Some(rest) = c.strip_prefix("gd-lint:") {
                let rest = rest.trim();
                if rest.starts_with("off") { off = true; break; }
                if let Some(list) = rest.strip_prefix("disable=") {
                    for item in list.split([',', ' ']).map(|s| s.trim()).filter(|s| !s.is_empty()) {
                        disabled.insert(item.to_string());
                    }
                }
                if let Some(val) = rest.strip_prefix("level=") {
                    let v = val.trim().to_lowercase();
                    level = match v.as_str() {
                        "info" => Some(Severity::Info),
                        "warn" | "warning" => Some(Severity::Warn),
                        "error" | "err" => Some(Severity::Error),
                        _ => level,
                    };
                }
            }
        }
    }
    (off, disabled, level)
}
