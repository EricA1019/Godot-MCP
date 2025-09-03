use std::fs;

#[test]
fn scene_issues_helper_reports_missing_script_and_extresource() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("project.godot"), "config_version=5\n").unwrap();

    let scene = r#"[gd_scene load_steps=2 format=2]

[ext_resource type="Script" path="res://scripts/missing.gd" id=1]

[node name="Root" type="Node" path="/root"]
script = ExtResource("1")
"#;
    fs::create_dir_all(root.join("scenes")).unwrap();
    fs::write(root.join("scenes/main.tscn"), scene).unwrap();

    let issues = godot_analyzer::scene_issues_as_report(root);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.message.contains("ExtResource(") || i.message.contains("Missing ext_resource path:")));
}
