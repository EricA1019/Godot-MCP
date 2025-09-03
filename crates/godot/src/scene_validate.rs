use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    let re_ext_line = Regex::new(r#"^\s*\[ext_resource\b"#).unwrap();
    let re_ext_id = Regex::new(r#"id\s*=\s*(\d+)"#).unwrap();
    let re_ext_path_attr = Regex::new(r#"path\s*=\s*\"([^\"]+)\""#).unwrap();
    let re_script_ext = Regex::new(r#"script\s*=\s*ExtResource\(\"(\d+)\"\)"#).unwrap();
    let re_prop_ext = Regex::new(r#"(?P<prop>[A-Za-z0-9_]+)\s*=\s*ExtResource\(\"(?P<id>\d+)\"\)"#).unwrap();
    let mut ext_map: HashMap<String, (String, usize)> = HashMap::new();
    for (i, line) in text.lines().enumerate() {
        let lno = i + 1;
        // ext_resource declarations
        if re_ext_line.is_match(line) {
            let id = re_ext_id
                .captures(line)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));
            let p = re_ext_path_attr
                .captures(line)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));
            if let (Some(id), Some(p)) = (id, p) {
                ext_map.insert(id.clone(), (p.clone(), lno));
                if let Some(res) = p.strip_prefix("res://") {
                    let target = root.join(res);
                    if !target.exists() {
                        out.push(SceneIssue { file: scene_rel.to_path_buf(), line: lno, node_path: None, message: format!("Missing ext_resource path: {}", p) });
                    }
                }
            }
            continue;
        }

        // track node path from [node ... path="..."] lines
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

        // script = ExtResource("id") form (explicit special-case for message clarity)
        if let Some(caps) = re_script_ext.captures(line) {
            let id = caps.get(1).unwrap().as_str();
            if let Some((path_str, _decl_line)) = ext_map.get(id) {
                if let Some(res) = path_str.strip_prefix("res://") {
                    let target = root.join(res);
                    if !target.exists() {
                        out.push(SceneIssue { file: scene_rel.to_path_buf(), line: lno, node_path: current_node_path.clone(), message: format!("Script ExtResource({}) missing file {}", id, path_str) });
                    }
                }
            } else {
                out.push(SceneIssue { file: scene_rel.to_path_buf(), line: lno, node_path: current_node_path.clone(), message: format!("Unknown ExtResource id: {}", id) });
            }
        }

        // Generic property = ExtResource("id") form
        if let Some(caps) = re_prop_ext.captures(line) {
            let prop = caps.name("prop").map(|m| m.as_str()).unwrap_or("");
            let id = caps.name("id").map(|m| m.as_str()).unwrap_or("");
            // If it's the script property, the script-specific handler above already covered it; skip duplicate
            if prop == "script" { continue; }
            if let Some((path_str, _decl_line)) = ext_map.get(id) {
                if let Some(res) = path_str.strip_prefix("res://") {
                    let target = root.join(res);
                    if !target.exists() {
                        out.push(SceneIssue { file: scene_rel.to_path_buf(), line: lno, node_path: current_node_path.clone(), message: format!("Property '{}' ExtResource({}) missing file {}", prop, id, path_str) });
                    }
                }
            } else {
                out.push(SceneIssue { file: scene_rel.to_path_buf(), line: lno, node_path: current_node_path.clone(), message: format!("Unknown ExtResource id: {}", id) });
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
