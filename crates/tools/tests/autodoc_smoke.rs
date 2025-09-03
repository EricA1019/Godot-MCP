#[test]
fn creates_or_updates_docs_in_tempdir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let report = tools::autodoc::ensure_autodocs(root).expect("ensure");
    assert!(report.created.len() + report.updated.len() + report.verified.len() >= 1);
}
