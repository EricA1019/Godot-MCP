use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneIssue {
    pub file: PathBuf,
    pub line: usize,
    pub node_path: Option<String>,
    pub message: String,
}

// Minimal validator: flags missing script for nodes with script="res://..." in .tscn
pub fn validate_scene(root: &Path, scene_rel: &Path) -> Vec<SceneIssue> {
    let path = root.join(scene_rel);
    let Ok(text) = fs::read_to_string(&path) else { return vec![] };
    let mut out = Vec::new();
    let mut current_node_path: Option<String> = None;
    for (i, line) in text.lines().enumerate() {
        let lno = i + 1;
        // track node path from [node name="Foo" parent=".." index=...] lines
        if line.trim_start().starts_with("[node ") {
            current_node_path = extract_attr(line, "path").map(|s| s.to_string());
        }
        // detect script attribute and resolve
        if let Some(script) = extract_attr(line, "script") {
            if let Some(p) = script.strip_prefix("res://") {
                let target = root.join(p);
                if !target.exists() {
                    out.push(SceneIssue {
                        file: scene_rel.to_path_buf(),
                        line: lno,
                        node_path: current_node_path.clone(),
                        message: format!("Missing script: {}", script),
                    });
                }
            }
        }
    }
    out
}

fn extract_attr<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    // naive parse: key="value" anywhere in line
    let pat = format!("{}=\"", key);
    let idx = line.find(&pat)? + pat.len();
    let rest = &line[idx..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}
