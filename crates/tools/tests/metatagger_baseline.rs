use std::fs;

#[test]
fn baseline_suppresses_matching_findings() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Seed a temp finding
    let p = root.join("foo.tmp");
    fs::write(&p, b"x").unwrap();

    // First run should detect it
    let r1 = tools::metatagger::run(root).unwrap();
    assert!(r1.findings.iter().any(|f| f.kind == "temp" && f.path.to_string_lossy() == "foo.tmp"));

    // Add baseline suppressing this finding
    let baseline = r#"[{"kind":"temp","path":"foo.tmp"}]"#;
    fs::write(root.join(".metatagger.baseline.json"), baseline).unwrap();

    // Second run should filter it out
    let r2 = tools::metatagger::run(root).unwrap();
    assert!(!r2.findings.iter().any(|f| f.kind == "temp" && f.path.to_string_lossy() == "foo.tmp"));
}
