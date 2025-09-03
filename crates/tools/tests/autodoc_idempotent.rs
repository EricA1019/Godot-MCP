#[test]
fn idempotent_creation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    // First run: should create files
    let rep1 = tools::autodoc::ensure_autodocs(root).expect("ensure1");
    assert!(!rep1.created.is_empty());

    // Second run: should verify, not recreate
    let rep2 = tools::autodoc::ensure_autodocs(root).expect("ensure2");
    assert!(rep2.created.is_empty());
    assert!(!rep2.verified.is_empty());
}
