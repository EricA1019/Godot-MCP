use std::fs;
use godot_analyzer::signal_issues_as_report;

#[test]
fn flags_missing_target_method_for_gdscript() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Create a simple script without the handler
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join("scripts/receiver.gd"), r#"
extends Node

func other_func():
    pass
"#).unwrap();

    let scene = r#"
[ext_resource type="Script" path="res://scripts/receiver.gd" id=1]

[node name="Root" type="Node" path="."]
[node name="A" type="Node" parent="."]
[node name="B" type="Node" parent="." script=ExtResource("1")]

[connection signal="pressed" from="A" to="B" method="on_pressed"]
"#;
    fs::write(root.join("test.tscn"), scene).unwrap();

    let issues = signal_issues_as_report(root);
    assert!(issues.iter().any(|i| i.message.starts_with("Target method not found:")));
}

#[test]
fn accepts_existing_target_method_for_gdscript() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join("scripts/receiver.gd"), r#"
extends Node

func on_pressed():
    pass
"#).unwrap();

    let scene = r#"
[ext_resource type="Script" path="res://scripts/receiver.gd" id=1]

[node name="Root" type="Node" path="."]
[node name="A" type="Node" parent="."]
[node name="B" type="Node" parent="." script=ExtResource("1")]

[connection signal="pressed" from="A" to="B" method="on_pressed"]
"#;
    fs::write(root.join("test.tscn"), scene).unwrap();

    let issues = signal_issues_as_report(root);
    assert!(!issues.iter().any(|i| i.message.starts_with("Target method not found:")));
}

#[test]
fn skips_method_check_for_csharp_script() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("scripts")).unwrap();
    // Create a dummy C# file reference via ext_resource
    fs::write(root.join("scripts/receiver.cs"), "// csharp placeholder").unwrap();

    let scene = r#"
[ext_resource type="Script" path="res://scripts/receiver.cs" id=1]

[node name="Root" type="Node" path="."]
[node name="A" type="Node" parent="."]
[node name="B" type="Node" parent="." script=ExtResource("1")]

[connection signal="pressed" from="A" to="B" method="on_pressed"]
"#;
    fs::write(root.join("test.tscn"), scene).unwrap();

    let issues = signal_issues_as_report(root);
    // Should not emit method-not-found for C# scripts
    assert!(!issues.iter().any(|i| i.message.starts_with("Target method not found:")));
}
