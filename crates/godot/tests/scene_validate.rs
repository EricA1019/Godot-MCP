use std::fs;

#[test]
fn reports_missing_script_in_scene() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // project root
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();
    // scene with a node referencing a missing script
    let scene = r#"[gd_scene load_steps=2 format=2]

[node name="Root" type="Node" path="/root"]
script="res://scripts/missing.gd"
"#;
    fs::create_dir_all(root.join("scenes")).unwrap();
    fs::write(root.join("scenes/main.tscn"), scene).unwrap();
    let issues = godot_analyzer::scene_validate::validate_scene(root, std::path::Path::new("scenes/main.tscn"));
    assert!(issues.iter().any(|i| i.message.contains("Missing script: res://scripts/missing.gd")));
    let hit = issues.iter().find(|i| i.message.contains("Missing script" )).unwrap();
    assert!(hit.line >= 3);
    assert_eq!(hit.node_path.as_deref(), Some("/root"));
}

#[test]
fn ok_when_script_exists() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join("scripts/exists.gd"), "extends Node\n").unwrap();
    let scene = r#"[gd_scene load_steps=2 format=2]

[node name="Root" type="Node" path="/root"]
script="res://scripts/exists.gd"
"#;
    fs::write(root.join("main.tscn"), scene).unwrap();
    let issues = godot_analyzer::scene_validate::validate_scene(root, std::path::Path::new("main.tscn"));
    assert!(issues.is_empty(), "expected no issues, got: {issues:?}");
}
