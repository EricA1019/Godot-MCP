use std::fs;

#[test]
fn warns_when_icon_and_main_scene_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // Minimal project.godot with no icon or main_scene
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();
    let report = godot_analyzer::analyze_project(root).unwrap();
    assert!(report.issues.iter().any(|i| i.message.contains("No application icon configured")));
    assert!(report.issues.iter().any(|i| i.message.contains("No main scene configured")));
}
