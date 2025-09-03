use std::fs;
use std::path::Path;
use tempfile::tempdir;

use godot_analyzer::structure_fix::{propose_plan, apply_plan, rollback_plan};

#[test]
fn propose_apply_and_rollback_plan_roundtrip() {
    let td = tempdir().unwrap();
    let root = td.path();

    // Seed files at repo root
    fs::write(root.join("a.gd"), "extends Node").unwrap();
    fs::write(root.join("b.tscn"), "[gd_scene]").unwrap();
    fs::write(root.join("c.png"), &[]).unwrap();

    // Existing target dirs shouldn't break
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::create_dir_all(root.join("scenes")).unwrap();
    fs::create_dir_all(root.join("images")).unwrap();

    let plan = propose_plan(root).expect("plan");
    // Expect 3 moves
    assert_eq!(plan.operations.len(), 3);

    // Paths are relative to root
    assert!(plan.operations.iter().any(|op| op.from == Path::new("a.gd") && op.to == Path::new("scripts/a.gd")));
    assert!(plan.operations.iter().any(|op| op.from == Path::new("b.tscn") && op.to == Path::new("scenes/b.tscn")));
    assert!(plan.operations.iter().any(|op| op.from == Path::new("c.png") && op.to == Path::new("images/c.png")));

    // Apply; ensure files moved
    let rollback = apply_plan(root, &plan).expect("apply");
    assert!(root.join("scripts/a.gd").exists());
    assert!(root.join("scenes/b.tscn").exists());
    assert!(root.join("images/c.png").exists());
    assert!(!root.join("a.gd").exists());
    assert!(!root.join("b.tscn").exists());
    assert!(!root.join("c.png").exists());

    // Rollback; ensure back in place
    rollback_plan(root, &rollback).expect("rollback");
    assert!(root.join("a.gd").exists());
    assert!(root.join("b.tscn").exists());
    assert!(root.join("c.png").exists());
}
