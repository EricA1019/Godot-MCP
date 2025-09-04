use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::scene_validate::SceneIssue;

/// Validate [connection] entries in a .tscn file.
/// - Checks that `from` and `to` node paths exist in the scene's node tree
/// - Checks presence of `signal` and `method` fields
/// - Flags duplicate connections (same signal/from/to/method)
pub fn validate_scene_signals(root: &Path, scene_rel: &Path) -> Vec<SceneIssue> {
    let path = root.join(scene_rel);
    let Ok(text) = fs::read_to_string(&path) else { return vec![] };
    let mut out = Vec::new();

    let re_node_line = Regex::new(r#"^\s*\[node\b"#).unwrap();
    let re_conn_line = Regex::new(r#"^\s*\[connection\b"#).unwrap();
    let re_ext_line = Regex::new(r#"^\s*\[ext_resource\b"#).unwrap();
    let re_ext_id = Regex::new(r#"id\s*=\s*(\d+)"#).unwrap();
    let re_ext_path_attr = Regex::new(r#"path\s*=\s*\"([^\"]+)\""#).unwrap();
    let re_script_ext = Regex::new(r#"script\s*=\s*ExtResource\(\"(\d+)\"\)"#).unwrap();
    let re_script_path = Regex::new(r#"script\s*=\s*\"(res://[^\"]+)\""#).unwrap();

    // First pass: build ext_resource id -> path map
    let mut ext_map: HashMap<String, String> = HashMap::new();
    for line in text.lines() {
        let line_trim = line.trim_start();
        if re_ext_line.is_match(line_trim) {
            let id = re_ext_id
                .captures(line_trim)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));
            let p = re_ext_path_attr
                .captures(line_trim)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));
            if let (Some(id), Some(p)) = (id, p) {
                ext_map.insert(id, p);
            }
        }
    }

    // Second pass: build known node paths and node->script mapping
    let mut node_paths: HashSet<String> = HashSet::new();
    node_paths.insert(".".to_string());
    let mut node_scripts: HashMap<String, String> = HashMap::new(); // node path -> res:// script path
    let mut current_node_path: Option<String> = None;
    let mut root_node_path: Option<String> = None;
    for line in text.lines() {
        let line_trim = line.trim_start();
        if re_node_line.is_match(line_trim) {
            // Entering a new node header line
            if let Some(p) = extract_attr(line_trim, "path") {
                let p = p.to_string();
                if root_node_path.is_none() && p == "." { root_node_path = Some(p.clone()); }
                node_paths.insert(p.clone());
                current_node_path = Some(p.clone());
                // Capture script attribute if present on the same line
                if let Some(caps) = re_script_ext.captures(line_trim) {
                    let id = caps.get(1).unwrap().as_str();
                    if let Some(path_str) = ext_map.get(id) {
                        if path_str.starts_with("res://") {
                            node_scripts.insert(p.clone(), path_str.clone());
                        }
                    }
                } else if let Some(caps) = re_script_path.captures(line_trim) {
                    let sp = caps.get(1).unwrap().as_str();
                    node_scripts.insert(p.clone(), sp.to_string());
                }
                continue;
            }
            let name = extract_attr(line_trim, "name");
            let parent = extract_attr(line_trim, "parent").unwrap_or(".");
            if let Some(n) = name {
                let full = if parent == "." { n.to_string() } else { format!("{}/{}", parent, n) };
                if root_node_path.is_none() && parent == "." { root_node_path = Some(full.clone()); }
                node_paths.insert(full.clone());
                current_node_path = Some(full.clone());
                // Capture script attribute if present on the same line
                if let Some(caps) = re_script_ext.captures(line_trim) {
                    let id = caps.get(1).unwrap().as_str();
                    if let Some(path_str) = ext_map.get(id) {
                        if path_str.starts_with("res://") {
                            node_scripts.insert(full.clone(), path_str.clone());
                        }
                    }
                } else if let Some(caps) = re_script_path.captures(line_trim) {
                    let sp = caps.get(1).unwrap().as_str();
                    node_scripts.insert(full.clone(), sp.to_string());
                }
            }
            continue;
        }
        // If inside a node block, attempt to capture script assignment
        if let Some(cur) = current_node_path.as_ref() {
            if let Some(caps) = re_script_ext.captures(line_trim) {
                let id = caps.get(1).unwrap().as_str();
                if let Some(path_str) = ext_map.get(id) {
                    if path_str.starts_with("res://") {
                        node_scripts.insert(cur.clone(), path_str.clone());
                    }
                }
            } else if let Some(caps) = re_script_path.captures(line_trim) {
                let p = caps.get(1).unwrap().as_str();
                node_scripts.insert(cur.clone(), p.to_string());
            }
        }
    }

    // detect duplicate connections
    let mut seen: HashMap<(String,String,String,String), usize> = HashMap::new();

    for (i, line) in text.lines().enumerate() {
        let lno = i + 1;
        let line_trim = line.trim_start();
        if !re_conn_line.is_match(line_trim) { continue; }

        let signal = extract_attr(line_trim, "signal");
        let from = extract_attr(line_trim, "from");
        let to = extract_attr(line_trim, "to");
        let method = extract_attr(line_trim, "method");

    if signal.is_none() { out.push(issue(scene_rel, lno, None, "Connection missing signal field — hint: set signal=\"<name>\" in [connection]")); }
    if method.is_none() { out.push(issue(scene_rel, lno, None, "Connection missing method field — hint: set method=\"<func>\" and ensure the target node's script defines it")); }

        if let Some(f) = from {
            if f != "." && !node_paths.contains(f) {
                out.push(issue(scene_rel, lno, None, &format!("Unknown connection 'from' node: {} — hint: create node or correct the 'from' path", f)));
            }
        } else {
            out.push(issue(scene_rel, lno, None, "Connection missing from field — hint: set from=\"<node_path>\" (use '.' for the scene root)"));
        }
        if let Some(t) = to {
            if t != "." && !node_paths.contains(t) {
                out.push(issue(scene_rel, lno, None, &format!("Unknown connection 'to' node: {} — hint: create node or correct the 'to' path", t)));
            }
        } else {
            out.push(issue(scene_rel, lno, None, "Connection missing to field — hint: set to=\"<node_path>\" (use '.' for the scene root)"));
        }

        if let (Some(s), Some(f), Some(t), Some(m)) = (signal, from, to, method) {
            let key = (s.to_string(), f.to_string(), t.to_string(), m.to_string());
            if let Some(_prev) = seen.insert(key.clone(), lno) {
                out.push(issue(scene_rel, lno, None, &format!("Duplicate connection: signal={} from={} to={} method={} — hint: remove the duplicate [connection] line", key.0, key.1, key.2, key.3)));
            }

            // Method existence checks (GDScript only)
            // Validate method name format first
            let method_name = m.trim();
            if method_name.is_empty() || !Regex::new(r#"^[A-Za-z_]\w*$"#).unwrap().is_match(method_name) {
                out.push(issue(scene_rel, lno, None, &format!("Invalid method name: '{}' — hint: use letters/numbers/underscore and start with a letter/underscore", m)));
            } else {
                // Resolve target node path -> script
                let target_node_lookup = if t == "." {
                    // Prefer explicit mapping for '.', otherwise use root node computed path
                    if node_scripts.contains_key(".") { Some(".".to_string()) } else { root_node_path.clone() }
                } else { Some(t.to_string()) };
                if let Some(tnp) = target_node_lookup {
                    if let Some(script_res_path) = node_scripts.get(&tnp) {
                        // Only check GDScript files
                        if script_res_path.ends_with(".gd") {
                            if let Some(res) = script_res_path.strip_prefix("res://") {
                                let script_fs_path = root.join(res);
                                if let Ok(src) = fs::read_to_string(&script_fs_path) {
                                    let pattern = format!(r#"(?m)^\s*func\s+{}\s*\("#, regex::escape(method_name));
                                    let re_func = Regex::new(&pattern).unwrap();
                                    if !re_func.is_match(&src) {
                                        out.push(issue(scene_rel, lno, None, &format!(
                                            "Target method not found: method='{}' to='{}' — hint: define 'func {}(...)' in {}",
                                            method_name, t, method_name, script_res_path
                                        )));
                                    }
                                }
                            }
                        } else {
                            // Non-GDScript (e.g., .cs or native) — skip method check
                        }
                    }
                }
            }
        }
    }

    out
}

fn extract_attr<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("{}=\"", key);
    let idx = line.find(&pat)? + pat.len();
    let rest = &line[idx..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn issue(scene_rel: &Path, line: usize, node_path: Option<String>, message: &str) -> SceneIssue {
    SceneIssue { file: scene_rel.to_path_buf(), line, node_path, message: message.to_string() }
}

// --- Graph (DOT) Export ---

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectionEdge {
    pub scene: PathBuf,
    pub from: String,
    pub to: String,
    pub signal: String,
    pub method: String,
}

/// Extract valid connections (with existing from/to nodes and present signal/method)
/// from a single .tscn scene file. Returns edges with scene-relative path.
pub fn extract_scene_connections(root: &Path, scene_rel: &Path) -> Vec<ConnectionEdge> {
    let path = root.join(scene_rel);
    let Ok(text) = fs::read_to_string(&path) else { return vec![] };

    let re_node_line = Regex::new(r#"^\s*\[node\b"#).unwrap();
    let re_conn_line = Regex::new(r#"^\s*\[connection\b"#).unwrap();

    // Build known node paths
    let mut node_paths: HashSet<String> = HashSet::new();
    node_paths.insert(".".to_string());
    for line in text.lines() {
        let line_trim = line.trim_start();
        if re_node_line.is_match(line_trim) {
            if let Some(p) = extract_attr(line_trim, "path") {
                node_paths.insert(p.to_string());
                continue;
            }
            let name = extract_attr(line_trim, "name");
            let parent = extract_attr(line_trim, "parent").unwrap_or(".");
            if let Some(n) = name {
                let full = if parent == "." { n.to_string() } else { format!("{}/{}", parent, n) };
                node_paths.insert(full);
            }
        }
    }

    // Collect well-formed connections
    let mut edges: Vec<ConnectionEdge> = Vec::new();
    for line in text.lines() {
        let line_trim = line.trim_start();
        if !re_conn_line.is_match(line_trim) { continue; }
        let signal = extract_attr(line_trim, "signal");
        let from = extract_attr(line_trim, "from");
        let to = extract_attr(line_trim, "to");
        let method = extract_attr(line_trim, "method");
        let (Some(s), Some(f), Some(t), Some(m)) = (signal, from, to, method) else { continue };
        if (f == "." || node_paths.contains(f)) && (t == "." || node_paths.contains(t)) {
            edges.push(ConnectionEdge {
                scene: scene_rel.to_path_buf(),
                from: f.to_string(),
                to: t.to_string(),
                signal: s.to_string(),
                method: m.to_string(),
            });
        }
    }
    // Deterministic ordering
    edges.sort();
    edges
}

/// Render a set of connection edges to a DOT graph (directed). Uses composite node ids
/// "<scene>:<node>" to avoid collisions. rankdir=LR for readability.
pub fn connections_to_dot(edges: &[ConnectionEdge]) -> String {
    fn esc<S: AsRef<str>>(s: S) -> String {
        s.as_ref().replace('\"', "\\\"")
    }
    let mut out = String::new();
    out.push_str("digraph Signals {\n");
    out.push_str("  rankdir=LR;\n");
    // Emit edges
    for e in edges {
        let sid = format!("{}:{}", e.scene.display(), e.from);
        let tid = format!("{}:{}", e.scene.display(), e.to);
        let label = format!("{}:{}", e.signal, e.method);
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            esc(sid), esc(tid), esc(label)
        ));
    }
    out.push_str("}\n");
    out
}

