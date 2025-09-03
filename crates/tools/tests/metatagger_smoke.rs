use std::fs;

#[test]
fn classifies_and_updates_cleanup_section() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Seed a couple of temp-like files
    let a = root.join("foo.tmp");
    fs::write(&a, b"x").unwrap();
    let b = root.join(".DS_Store");
    fs::write(&b, b"x").unwrap();

    // Run
    let report = tools::metatagger::run(root).unwrap();
    assert!(report.findings.len() >= 1);

    // PROJECT_INDEX must exist and contain cleanup region
    let proj = root.join("docs/PROJECT_INDEX.md");
    let content = fs::read_to_string(&proj).unwrap();
    assert!(content.contains("<!-- METATAGGER:BEGIN cleanup -->"));
    assert!(content.contains("<!-- METATAGGER:END cleanup -->"));
}

#[test]
fn respects_ignores_and_detects_duplicates() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Ignored file pattern
    fs::write(root.join(".metataggerignore"), "ignored/**\n").unwrap();
    fs::create_dir_all(root.join("ignored")).unwrap();
    fs::write(root.join("ignored/file.tmp"), b"x").unwrap();

    // Duplicate content
    fs::write(root.join("a.bin"), b"same").unwrap();
    fs::write(root.join("b.bin"), b"same").unwrap();

    let report = tools::metatagger::run(root).unwrap();

    // Ensure ignored temp didn't produce findings
    assert!(report.findings.iter().all(|f| !f.path.to_string_lossy().contains("ignored/file.tmp")));

    // Duplicates should be reported for both files
    let dup_count = report.findings.iter().filter(|f| f.kind == "duplicate").count();
    assert!(dup_count >= 2);
}
