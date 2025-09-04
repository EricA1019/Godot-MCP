use std::fs;
use godot_analyzer::signal_graph_dot;

#[test]
fn generates_signal_graph_dot() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let scene = r#"
[node name="Root" type="Node"]
[node name="A" type="Node" parent="."]
[node name="B" type="Node" parent="."]

[connection signal="pressed" from="A" to="B" method="on_pressed"]
[connection signal="released" from="B" to="A" method="on_released"]
"#;
    let p = root.join("test.tscn");
    fs::write(&p, scene).unwrap();

    let dot = signal_graph_dot(root);
    // Basic shape checks
    assert!(dot.starts_with("digraph Signals"));
    assert!(dot.contains("\"test.tscn:A\" -> \"test.tscn:B\" [label=\"pressed:on_pressed\"];"));
    assert!(dot.contains("\"test.tscn:B\" -> \"test.tscn:A\" [label=\"released:on_released\"];"));
}
