use std::fs; use godot_analyzer::signal_issues_as_report;

#[test]
fn flags_unknown_from_to_and_duplicate_connection() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let scene = r#"
[node name="Root" type="Node"]
[node name="A" type="Node" parent="."]

[connection signal="pressed" from="A" to="X" method="on_pressed"]
[connection signal="pressed" from="A" to="X" method="on_pressed"]
"#;
    let p = root.join("test.tscn");
    fs::write(&p, scene).unwrap();

    let issues = signal_issues_as_report(root);
    // Should flag unknown 'to' node and duplicate connection
    assert!(issues.iter().any(|i| i.message.starts_with("Unknown connection 'to' node:")));
    assert!(issues.iter().any(|i| i.message.starts_with("Duplicate connection:")));
}
