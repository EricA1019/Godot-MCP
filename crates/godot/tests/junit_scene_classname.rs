use std::fs;

#[test]
fn junit_uses_scene_validator_classname_for_scene_findings() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();

    // Create a scene with a missing script via ExtResource
    let scene = r#"[gd_scene load_steps=2 format=2]

[ext_resource type="Script" path="res://scripts/missing.gd" id=1]

[node name="Root" type="Node" path="/root"]
script = ExtResource("1")
"#;
    fs::write(root.join("main.tscn"), scene).unwrap();

    let mut report = godot_analyzer::analyze_project(root).unwrap();
    let scene_issues = godot_analyzer::scene_issues_as_report(root);
    assert!(!scene_issues.is_empty());
    report.issues.extend(scene_issues);
    let junit = godot_analyzer::to_junit(&report);
    assert!(junit.contains("classname=\"scene-validator\""));
}
