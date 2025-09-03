use std::fs;

#[test]
fn detects_missing_ext_resource() {
    let root = tempfile::tempdir().unwrap();
    let rootp = root.path();

    // Minimal project.godot
    fs::write(rootp.join("project.godot"), "config_version=5\n").unwrap();

    // Scene referencing a non-existent resource
    fs::create_dir_all(rootp.join("scenes")).unwrap();
    let scene = r#"[gd_scene load_steps=2 format=3]
[ext_resource type="Texture2D" uid="uid://abcd" path="res://assets/missing.png" id="1"]
[node name="Root" type="Node"]
"#;
    fs::write(rootp.join("scenes/missing_ref.tscn"), scene).unwrap();

    let report = godot_analyzer::analyze_project(rootp).unwrap();
    assert!(report.issues.iter().any(|i| i.message.contains("Missing ext_resource path")));
}
