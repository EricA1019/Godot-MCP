use std::fs;

#[test]
fn unknown_subresource_id_is_reported() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();

    // Scene references a SubResource that isn't declared
    let scene = r#"[gd_scene load_steps=2 format=2]

[node name="Root" type="AnimationPlayer" path="/root"]
libraries = {"": SubResource("1")}
"#;
    fs::write(root.join("main.tscn"), scene).unwrap();
    let issues = godot_analyzer::scene_validate::validate_scene(root, std::path::Path::new("main.tscn"));
    assert!(issues.iter().any(|i| i.message.contains("Unknown SubResource id: 1")), "issues: {issues:?}");
}

#[test]
fn preload_missing_file_is_reported() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();

    // preload() within a script line referencing a missing file
    let scene = r#"[gd_scene load_steps=2 format=2]

[node name="Root" type="Node" path="/root"]
script="res://scripts/exists.gd"
# GDScript-like content embedded or adjacent lines
_some_prop = preload("res://not_found/thing.tscn")
"#;
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join("scripts/exists.gd"), "extends Node\n").unwrap();
    fs::write(root.join("main.tscn"), scene).unwrap();
    let issues = godot_analyzer::scene_validate::validate_scene(root, std::path::Path::new("main.tscn"));
    assert!(issues.iter().any(|i| i.message.contains("Preload missing file: res://not_found/thing.tscn")), "issues: {issues:?}");
}

#[test]
fn load_missing_file_is_reported() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();

    let scene = r#"[gd_scene load_steps=2 format=2]

[node name="Root" type="Node" path="/root"]
_some_prop = load("res://not_found/thing2.tscn")
"#;
    fs::write(root.join("main.tscn"), scene).unwrap();
    let issues = godot_analyzer::scene_validate::validate_scene(root, std::path::Path::new("main.tscn"));
    assert!(issues.iter().any(|i| i.message.contains("Load missing file: res://not_found/thing2.tscn")), "issues: {issues:?}");
}
