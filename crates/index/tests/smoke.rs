use std::fs;
use index::{IndexPaths, SearchIndex};

#[test]
fn scan_and_query_smoke() {
    // Setup temp dirs
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("root");
    let data = tmp.path().join("data");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&data).unwrap();

    // Create a couple files
    fs::write(root.join("a.rs"), "fn main() { println!(\"godot\"); }").unwrap();
    fs::write(root.join("b.md"), "This mentions Godot engine.").unwrap();

    let paths = IndexPaths { root: root.clone(), data_dir: data.clone() };
    let mut idx = SearchIndex::open(&paths).unwrap();
    let n = idx.scan_and_index(&root).unwrap();
    assert!(n >= 2);

    let hits = idx.query("godot", 10).unwrap();
    assert!(!hits.is_empty());
}
