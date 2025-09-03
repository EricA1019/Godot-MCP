use std::fs;

#[test]
fn property_extresource_missing_file_is_reported_and_ruleid_is_scene_validator() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();

    // Declare a texture as ext resource, then assign to a property
    let scene = r#"[gd_scene load_steps=2 format=2]

[ext_resource type="Texture2D" path="res://textures/missing.png" id=1]

[node name="Root" type="Sprite2D" path="/root"]
texture = ExtResource("1")
"#;
    fs::write(root.join("main.tscn"), scene).unwrap();

    // Scene-level validation catches the property assignment
    let issues = godot_analyzer::scene_validate::validate_scene(root, std::path::Path::new("main.tscn"));
    assert!(issues.iter().any(|i| i.message.contains("Property 'texture' ExtResource(1) missing file res://textures/missing.png")));

    // Wire into report issues to build SARIF with ruleIds
    let mut report = godot_analyzer::analyze_project(root).unwrap();
    let scene_issues = godot_analyzer::scene_issues_as_report(root);
    assert!(!scene_issues.is_empty());
    report.issues.extend(scene_issues);
    let sarif = godot_analyzer::to_sarif(&report);
    let s = serde_json::to_string(&sarif).unwrap();
    assert!(s.contains("scene-validator"));
}
