use std::fs;
use godot_analyzer::structure_fix::{plan_structure_fix, apply_structure_fix};

#[test]
fn apply_moves_and_updates_references() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Seed project files
    fs::write(root.join("player.gd"), "extends Node\nvar T = preload(\"res://player.gd\")\n").unwrap();
    fs::write(root.join("main.tscn"), "[gd_scene]\n[ext_resource path=\"res://player.gd\" type=\"Script\" id=1]\n[node name=Root type=Node]\n").unwrap();
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("sub/tex.png"), "fake").unwrap();

    let plan = plan_structure_fix(root);
    assert!(!plan.moves.is_empty());
    let sum = apply_structure_fix(root, &plan).expect("apply");
    assert!(!sum.moved.is_empty());
    // Files moved
    assert!(root.join("scripts/player.gd").exists());
    assert!(root.join("scenes/main.tscn").exists());
    assert!(root.join("assets/sub/tex.png").exists());
    // Backups exist
    assert!(root.join(".structure_fix/backup/player.gd").exists());
    assert!(root.join(".structure_fix/backup/main.tscn").exists());
    assert!(root.join(".structure_fix/backup/sub/tex.png").exists());

    // References updated
    let scene = fs::read_to_string(root.join("scenes/main.tscn")).unwrap();
    assert!(scene.contains("res://scripts/player.gd"));
    let gd = fs::read_to_string(root.join("scripts/player.gd")).unwrap();
    assert!(gd.contains("res://scripts/player.gd"));
}
