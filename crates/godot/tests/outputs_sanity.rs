use std::fs;

#[test]
fn sarif_and_junit_generation_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("project.godot"), "[application]\nconfig_version=5\n").unwrap();
    let report = godot_analyzer::analyze_project(root).unwrap();
    let sarif = godot_analyzer::to_sarif(&report);
    let s = serde_json::to_string(&sarif).unwrap();
    assert!(s.contains("\"runs\""));
    let junit = godot_analyzer::to_junit(&report);
    assert!(junit.contains("<testsuite"));
}
