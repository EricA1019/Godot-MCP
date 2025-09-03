use regex::Regex;
use std::collections::{HashSet, HashMap};
use std::fs;
use std::path::Path;

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

    let mut node_paths: HashSet<String> = HashSet::new();
    node_paths.insert(".".to_string());

    // build known node paths: prefer explicit path="..."; otherwise derive from name+parent
    for (_i, line) in text.lines().enumerate() {
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

        if signal.is_none() { out.push(issue(scene_rel, lno, None, "Connection missing signal field")); }
        if method.is_none() { out.push(issue(scene_rel, lno, None, "Connection missing method field")); }

        if let Some(f) = from {
            if f != "." && !node_paths.contains(f) {
                out.push(issue(scene_rel, lno, None, &format!("Unknown connection 'from' node: {}", f)));
            }
        } else {
            out.push(issue(scene_rel, lno, None, "Connection missing from field"));
        }
        if let Some(t) = to {
            if t != "." && !node_paths.contains(t) {
                out.push(issue(scene_rel, lno, None, &format!("Unknown connection 'to' node: {}", t)));
            }
        } else {
            out.push(issue(scene_rel, lno, None, "Connection missing to field"));
        }

        if let (Some(s), Some(f), Some(t), Some(m)) = (signal, from, to, method) {
            let key = (s.to_string(), f.to_string(), t.to_string(), m.to_string());
            if let Some(_prev) = seen.insert(key.clone(), lno) {
                out.push(issue(scene_rel, lno, None, &format!("Duplicate connection: signal={} from={} to={} method={}", key.0, key.1, key.2, key.3)));
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
