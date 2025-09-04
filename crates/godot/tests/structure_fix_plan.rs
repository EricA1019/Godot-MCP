use std::fs;
use godot_analyzer::structure_fix::plan_structure_fix;

#[test]
fn plans_moves_for_scripts_and_scenes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("subdir")).unwrap();
    fs::write(root.join("main.tscn"), "[node name=Root type=Node]").unwrap();
    fs::write(root.join("player.gd"), "extends Node").unwrap();
    fs::write(root.join("subdir/tex.png"), "fake").unwrap();

    let plan = plan_structure_fix(root);
    let moves: Vec<(String,String)> = plan.moves.iter()
        .map(|m| (m.from.to_string_lossy().to_string(), m.to.to_string_lossy().to_string()))
        .collect();

    assert!(moves.contains(&("res://player.gd".into(), "res://scripts/player.gd".into())));
    assert!(moves.contains(&("res://main.tscn".into(), "res://scenes/main.tscn".into())));
    assert!(moves.contains(&("res://subdir/tex.png".into(), "res://assets/subdir/tex.png".into())));
}
