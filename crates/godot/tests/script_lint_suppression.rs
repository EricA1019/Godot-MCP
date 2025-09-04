use std::fs;
use godot_analyzer::script_lint::lint_gd_scripts;

#[test]
fn respects_gd_lint_disable_list() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // This file has multiple issues but disables two of them
    fs::write(
        root.join("Suppressed.gd"),
        r#"# gd-lint: disable=debug-print,tab-indentation
class_name Foo
print("dbg")
	# tab indent
extends Node
"#,
    )
    .unwrap();

    let findings = lint_gd_scripts(root);
    let msgs: Vec<String> = findings.iter().map(|f| f.message.clone()).collect();

    // Suppressed ones should not appear
    assert!(!msgs.iter().any(|m| m == "Debug print found"));
    assert!(!msgs.iter().any(|m| m == "Tab indentation used"));

    // Non-suppressed should remain (class_name mismatch)
    assert!(msgs.iter().any(|m| m.starts_with("Class name mismatch:")));
}

#[test]
fn respects_gd_lint_off() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::write(
        root.join("DisabledAll.gd"),
        r#"# gd-lint: off
print("dbg")
	# tab indent
"#,
    )
    .unwrap();

    let findings = lint_gd_scripts(root);
    assert!(findings.is_empty());
}
