use std::path::Path;

#[test]
fn creates_or_verifies_docs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    let report = tools::autodoc::ensure_autodocs(&root).expect("ensure");
    assert!(report.created.len() + report.verified.len() >= 1);
}
