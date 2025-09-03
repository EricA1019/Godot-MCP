fn find_repo_root(mut p: std::path::PathBuf) -> std::path::PathBuf {
    loop {
        if p.join("project.godot").exists() { return p; }
        if !p.pop() { break; }
    }
    panic!("could not find project.godot in ancestors");
}

#[test]
fn smoke_analyze_current_repo() {
    let cwd = std::env::current_dir().unwrap();
    let root = find_repo_root(cwd);
    let report = godot_analyzer::analyze_project(&root).unwrap();
    assert_eq!(report.project_path, root);
    // Should detect our addons folder at workspace root
    assert!(report.addons.len() >= 1, "expected addons in {}", root.display());
}
