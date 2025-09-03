use std::fs;

#[test]
fn warns_when_addon_missing_plugin_cfg() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("addons/foo")) .unwrap();
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();
    let report = godot_analyzer::analyze_project(root).unwrap();
    assert!(report.issues.iter().any(|i| i.message.contains("Addon 'foo' missing plugin.cfg")));
}
