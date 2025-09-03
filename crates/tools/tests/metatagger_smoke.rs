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
