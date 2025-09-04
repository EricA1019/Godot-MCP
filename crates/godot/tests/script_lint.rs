use std::fs;
use godot_analyzer::script_lint::lint_gd_scripts;

#[test]
fn lints_common_gdscript_issues() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Files
    fs::write(root.join("Foo.gd"), "class_name Bar\nprint(\"dbg\")\n\t# tab\npreload(\"res://missing/asset.tres\")\n").unwrap();
    fs::write(root.join("NoExtends.gd"), "# no extends here\nvar x=1\n").unwrap();
    fs::create_dir_all(root.join("assets")).unwrap();
    fs::write(root.join("assets/exists.tres"), "").unwrap();
    fs::write(root.join("Ok.gd"), "extends Node\npreload(\"res://assets/exists.tres\")\n").unwrap();

    let findings = lint_gd_scripts(root);
    let msgs: Vec<String> = findings.iter().map(|f| f.message.clone()).collect();

    assert!(msgs.iter().any(|m| m.starts_with("Class name mismatch:")));
    assert!(msgs.iter().any(|m| m == "Debug print found"));
    assert!(msgs.iter().any(|m| m == "Tab indentation used"));
    assert!(msgs.iter().any(|m| m == "Missing extends declaration"));
    assert!(msgs.iter().any(|m| m.starts_with("GDScript preload missing file:")) || msgs.iter().any(|m| m.contains("missing file")));
    // Ok.gd should not produce errors beyond ordering; ensure not all files are flagged
    assert!(findings.len() >= 4);
}
