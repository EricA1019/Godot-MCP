use std::{fs, time::Instant};
use tempfile::tempdir;
use index::{IndexPaths, SearchIndex};

#[test]
fn add_update_delete_updates_index() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let data_dir = root.join(".index_data");
    let paths = IndexPaths { root: root.clone(), data_dir };

    // Create a file
    let file_path = root.join("file.txt");
    fs::write(&file_path, "hello world").unwrap();

    // Open index and initial scan
    let mut idx = SearchIndex::open(&paths).unwrap();
    let _ = idx.scan_and_index(&root).unwrap();
    let hits = idx.query("world", 5).unwrap();
    assert!(hits.iter().any(|(_, p)| p.ends_with("file.txt")));

    // Update file content
    fs::write(&file_path, "hello rust").unwrap();
    let t0 = Instant::now();
    idx.apply_batch(&[], &[file_path.clone()]).unwrap();
    let dt = t0.elapsed();
    // Basic perf sanity check (avoid flakiness on slow CI): <250ms per file
    assert!(dt.as_millis() < 250, "apply_batch took {:?}", dt);

    // Ensure new content is searchable, old is not
    let hits_new = idx.query("rust", 5).unwrap();
    assert!(hits_new.iter().any(|(_, p)| p.ends_with("file.txt")));
    // Trigger a refresh and re-query to ensure visibility of latest commit
    let _ = idx.query("", 1); // no-op refresh
    let hits_old = idx.query("world", 5).unwrap();
    assert!(!hits_old.iter().any(|(_, p)| p.ends_with("file.txt")));

    // Delete file and propagate
    fs::remove_file(&file_path).unwrap();
    idx.apply_batch(&[file_path.clone()], &[]).unwrap();
    let hits_after_delete = idx.query("rust", 5).unwrap();
    assert!(!hits_after_delete.iter().any(|(_, p)| p.ends_with("file.txt")));
}
