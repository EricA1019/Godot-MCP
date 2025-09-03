// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ Crate: index                                                        ┃
// ┃ Purpose: Full-text index (Tantivy) with FS watcher + CLI            ┃
// ┃ Author: EricA1019                                                   ┃
// ┃ Last Updated: 2025-09-02                                           ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use anyhow::Result;
use std::{fs, path::{Path, PathBuf}};
use tantivy::{collector::TopDocs, doc, schema::{Field, Schema, SchemaBuilder, TEXT, STORED, STRING}, Index, IndexWriter};
// (no ReloadPolicy needed with fresh readers per query)
use tantivy::query::{BooleanQuery, Occur, Query, TermQuery};
use tantivy::Term;
use tracing::{info, warn};
use walkdir::WalkDir;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, EventKind};
use std::sync::mpsc::channel;
use xxhash_rust::xxh3::xxh3_64;
use std::time::Duration;
use std::collections::HashSet;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;

#[derive(Clone)]
pub struct IndexPaths {
    pub root: PathBuf,
    pub data_dir: PathBuf,
}

pub struct SearchIndex {
    index: Index,
    writer: IndexWriter,
    fields: Fields,
    root: PathBuf,
}

#[derive(Clone, Copy)]
struct Fields { path: Field, content: Field, kind: Field, hash: Field }

pub fn build_schema() -> Schema {
    let mut builder = SchemaBuilder::default();
    let _path = builder.add_text_field("path", STRING | STORED);
    // Store content to enable optional snippets in responses
    let _content = builder.add_text_field("content", TEXT | STORED);
    let _kind = builder.add_text_field("kind", STRING | STORED);
    let _hash = builder.add_text_field("hash", STRING | STORED);
    builder.build()
}

fn detect_kind(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "rust",
        Some("gd") => "gdscript",
        Some("tscn") | Some("tres") => "godot",
        Some("md") => "docs",
        Some("toml") | Some("yaml") | Some("yml") | Some("json") => "config",
        _ => "other",
    }
}

impl SearchIndex {
    pub fn open(paths: &IndexPaths) -> Result<Self> {
        fs::create_dir_all(&paths.data_dir)?;
        let schema = build_schema();
    let mmap_dir = tantivy::directory::MmapDirectory::open(&paths.data_dir)?;
    let mut index = match Index::open_or_create(mmap_dir, schema.clone()) {
        Ok(idx) => idx,
        Err(e) => {
            warn!(error=%e, "Index open_or_create failed, recreating index directory");
            // Attempt to rebuild index directory (schema changes etc.)
            let _ = fs::remove_dir_all(&paths.data_dir);
            fs::create_dir_all(&paths.data_dir)?;
            let mmap_dir = tantivy::directory::MmapDirectory::open(&paths.data_dir)?;
            Index::open_or_create(mmap_dir, schema.clone())?
        }
    };
    let writer = index.writer(50_000_000)?; // 50MB
        let fields = Fields {
            path: index.schema().get_field("path").unwrap(),
            content: index.schema().get_field("content").unwrap(),
            kind: index.schema().get_field("kind").unwrap(),
            hash: index.schema().get_field("hash").unwrap(),
        };
    let _ = index.set_default_multithread_executor();
        // Canonicalize root for consistent normalization
        let root = paths.root.canonicalize().unwrap_or(paths.root.clone());
    Ok(Self { index, writer, fields, root })
    }

    fn normalize_path(&self, path: &Path) -> String {
        let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if let Ok(rel) = abs.strip_prefix(&self.root) {
            // Ensure leading ./ for relative consistency
            format!("./{}", rel.display())
        } else {
            abs.to_string_lossy().to_string()
        }
    }

    /// Convert a normalized index path (e.g., "./rel/path") back to an absolute PathBuf using the index root.
    pub fn absolutize_path(&self, normalized: &str) -> PathBuf {
        let p = std::path::Path::new(normalized);
    if let Some(stripped) = normalized.strip_prefix("./") {
            return self.root.join(stripped);
        }
        if p.is_relative() {
            return self.root.join(p);
        }
        p.to_path_buf()
    }

    pub fn scan_and_index(&mut self, root: &Path) -> Result<usize> {
        let mut count = 0usize;
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() { continue; }
            let path = entry.path();
            // If file matches skip rules, ensure any previously indexed doc is removed
            if should_skip(path) {
                let path_str = self.normalize_path(path);
                let _ = self.writer.delete_term(Term::from_field_text(self.fields.path, &path_str));
                continue;
            }
            count += self.index_file(path).unwrap_or(0);
        }
    let _ = self.writer.commit()?;
        Ok(count)
    }

    pub fn index_file(&mut self, path: &Path) -> Result<usize> {
        let content = fs::read_to_string(path).unwrap_or_default();
        let kind = detect_kind(path);
        let hash = format!("{:x}", xxh3_64(content.as_bytes()));
    let path_str = self.normalize_path(path);

    // Ensure only one doc per path by deleting any existing doc for this path first
    let _ = self.writer.delete_term(Term::from_field_text(self.fields.path, &path_str));

    let _ = self.writer.add_document(doc!(
            self.fields.path => path_str,
            self.fields.content => content,
            self.fields.kind => kind.to_string(),
            self.fields.hash => hash,
        ));
        Ok(1)
    }

    pub fn query(&self, q: &str, limit: usize) -> Result<Vec<(f32, String)>> {
        let q = q.trim();
        if q.is_empty() { return Ok(vec![]); }
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        // Build AND-of-terms query over the content field
        let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        for term in q.split_whitespace().filter(|s| !s.is_empty()) {
            let tq = TermQuery::new(Term::from_field_text(self.fields.content, term), tantivy::schema::IndexRecordOption::Basic);
            clauses.push((Occur::Must, Box::new(tq)));
        }
        if clauses.is_empty() { return Ok(vec![]); }
        let query: Box<dyn Query> = if clauses.len() == 1 {
            clauses.pop().unwrap().1
        } else {
            Box::new(BooleanQuery::new(clauses))
        };

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        let mut hits = Vec::new();
        for (score, addr) in top_docs {
            let doc_map = searcher.doc::<std::collections::HashMap<Field, tantivy::schema::document::OwnedValue>>(addr)?;
            if let Some(tantivy::schema::document::OwnedValue::Str(path_str)) = doc_map.get(&self.fields.path) {
                hits.push((score, path_str.clone()));
            }
        }
        Ok(hits)
    }

    /// Apply a batch of deletions and (re)indexing in a single commit.
    /// Skips files matching internal skip rules.
    pub fn apply_batch(&mut self, to_delete: &[PathBuf], to_index: &[PathBuf]) -> Result<()> {
    // Apply deletions first
        for p in to_delete.iter() {
            if should_skip(p) { continue; }
            let path_str = self.normalize_path(p);
            let _ = self.writer.delete_term(Term::from_field_text(self.fields.path, &path_str));
        }
    // Commit deletions so they are visible to searchers before re-adding updated docs
    self.writer.commit()?;

    // Then apply (re)indexing; avoid duplicates where a path is both deleted and indexed
        let del_set: HashSet<&PathBuf> = to_delete.iter().collect();
        for p in to_index.iter() {
            if del_set.contains(p) { continue; }
            if should_skip(p) { continue; }
            let _ = self.index_file(p);
        }
    self.writer.commit()?;
        Ok(())
    }

    /// Advanced query with optional kind filtering and optional snippet extraction.
    pub fn query_filtered(
        &self,
        q: &str,
        kind: Option<&str>,
        limit: usize,
        with_snippet: bool,
    ) -> Result<Vec<(f32, String, String, Option<String>)>> {
    // Use a fresh reader to ensure we always see the latest committed data
    let reader = self.index.reader()?;
    let searcher = reader.searcher();

        // Build content query
        let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        if !q.trim().is_empty() {
            let mut inner: Vec<(Occur, Box<dyn Query>)> = Vec::new();
            for term in q.split_whitespace().filter(|s| !s.is_empty()) {
                let tq = TermQuery::new(Term::from_field_text(self.fields.content, term), tantivy::schema::IndexRecordOption::Basic);
                inner.push((Occur::Must, Box::new(tq)));
            }
            if inner.len() == 1 {
                clauses.push(inner.pop().unwrap());
            } else if !inner.is_empty() {
                clauses.push((Occur::Must, Box::new(BooleanQuery::new(inner))));
            }
        }
        // Optional kind filter as exact term query
        if let Some(k) = kind {
            let term = Term::from_field_text(self.fields.kind, k);
            clauses.push((Occur::Must, Box::new(TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic))));
        }

        let query: Box<dyn Query> = if clauses.is_empty() {
            // Match nothing if no query provided
            Box::new(BooleanQuery::new(vec![]))
        } else if clauses.len() == 1 {
            clauses.pop().unwrap().1
        } else {
            Box::new(BooleanQuery::new(clauses))
        };

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        let mut hits = Vec::new();
        for (score, addr) in top_docs {
            let doc_map = searcher.doc::<std::collections::HashMap<Field, tantivy::schema::document::OwnedValue>>(addr)?;
            let path = match doc_map.get(&self.fields.path) {
                Some(tantivy::schema::document::OwnedValue::Str(s)) => s.clone(),
                _ => continue,
            };
            let kind_val = match doc_map.get(&self.fields.kind) {
                Some(tantivy::schema::document::OwnedValue::Str(s)) => s.clone(),
                _ => "".to_string(),
            };
            let snippet = if with_snippet {
                match doc_map.get(&self.fields.content) {
                    Some(tantivy::schema::document::OwnedValue::Str(c)) => Some(make_snippet(c, q)),
                    _ => None,
                }
            } else { None };
            hits.push((score, path, kind_val, snippet));
        }
        Ok(hits)
    }

    /// Lightweight health info: (doc_count, segment_count)
    pub fn health(&self) -> Result<(u64, usize)> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let doc_count = searcher.num_docs() as u64;
        let segments = searcher.segment_readers().len();
        Ok((doc_count, segments))
    }

    /// Watch the filesystem under root and incrementally index changes.
    /// Blocks the current thread.
    pub fn watch(&mut self, root: &Path) -> Result<()> {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, notify::Config::default())?;
        watcher.watch(root, RecursiveMode::Recursive)?;
        info!("Starting index watcher on {}", root.display());

        loop {
            // Block for the first event
            let evt = match rx.recv() {
                Ok(Ok(e)) => e,
                Ok(Err(e)) => { warn!(error=%e, "watch error"); continue; },
                Err(e) => { warn!(error=%e, "recv error"); continue; },
            };

            let mut to_index: HashSet<PathBuf> = HashSet::new();
            let mut to_delete: HashSet<PathBuf> = HashSet::new();

            let mut push_event = |event_kind: &EventKind, paths: &Vec<PathBuf>| {
                match event_kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        for p in paths {
                            if p.is_file() && !should_skip(p) { to_index.insert(p.clone()); }
                        }
                    }
                    EventKind::Remove(_) => {
                        for p in paths {
                            if !should_skip(p) { to_delete.insert(p.clone()); }
                        }
                    }
                    _ => {}
                }
            };

            push_event(&evt.kind, &evt.paths);

            // Debounce window: accumulate events for a short period
            while let Ok(res) = rx.recv_timeout(Duration::from_millis(200)) {
                match res {
                    Ok(e) => push_event(&e.kind, &e.paths),
                    Err(e) => { warn!(error=%e, "watch error"); break; }
                }
            }

            // Apply deletions first
            for p in to_delete.iter() {
                let path_str = self.normalize_path(p);
                let _ = self.writer.delete_term(Term::from_field_text(self.fields.path, &path_str));
            }
            // Then apply (re)indexing; skip any files that were also deleted in this batch
            for p in to_index.into_iter() {
                if to_delete.contains(&p) { continue; }
                let _ = self.index_file(&p);
            }

            let _ = self.writer.commit();
        }
    }

    /// Same as `watch` but allows cooperative shutdown via an AtomicBool.
    /// When `shutdown` is set to true, the watcher will stop shortly after.
    pub fn watch_with_shutdown(&mut self, root: &Path, shutdown: Arc<AtomicBool>) -> Result<()> {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, notify::Config::default())?;
        watcher.watch(root, RecursiveMode::Recursive)?;
        info!("Starting index watcher on {} (with shutdown)", root.display());

        'outer: loop {
            if shutdown.load(Ordering::Relaxed) { break; }
            // Block for the first event with a timeout so we can observe shutdown
            let evt = match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(Ok(e)) => e,
                Ok(Err(e)) => { warn!(error=%e, "watch error"); continue; },
                Err(RecvTimeoutError::Timeout) => { continue; },
                Err(e) => { warn!(error=%e, "recv error"); continue; },
            };

            let mut to_index: HashSet<PathBuf> = HashSet::new();
            let mut to_delete: HashSet<PathBuf> = HashSet::new();

            let mut push_event = |event_kind: &EventKind, paths: &Vec<PathBuf>| {
                match event_kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        for p in paths {
                            if p.is_file() && !should_skip(p) { to_index.insert(p.clone()); }
                        }
                    }
                    EventKind::Remove(_) => {
                        for p in paths {
                            if !should_skip(p) { to_delete.insert(p.clone()); }
                        }
                    }
                    _ => {}
                }
            };

            push_event(&evt.kind, &evt.paths);

            // Debounce window: accumulate events for a short period
            while !shutdown.load(Ordering::Relaxed) {
                match rx.recv_timeout(Duration::from_millis(200)) {
                    Ok(Ok(e)) => push_event(&e.kind, &e.paths),
                    Ok(Err(e)) => { warn!(error=%e, "watch error"); break; },
                    Err(RecvTimeoutError::Timeout) => { break; },
                    Err(e) => { warn!(error=%e, "recv error"); break; },
                }
            }

            if shutdown.load(Ordering::Relaxed) { break 'outer; }

            // Apply deletions first
            for p in to_delete.iter() {
                let path_str = self.normalize_path(p);
                let _ = self.writer.delete_term(Term::from_field_text(self.fields.path, &path_str));
            }
            // Then apply (re)indexing; skip any files that were also deleted in this batch
            for p in to_index.into_iter() {
                if to_delete.contains(&p) { continue; }
                let _ = self.index_file(&p);
            }

            let _ = self.writer.commit();
        }
        info!("Index watcher shutdown complete");
        Ok(())
    }

    fn rescan(&mut self, root: &Path) -> Result<()> {
        // Create a new writer (simple approach) and rescan
        self.writer.rollback()?;
        let _ = self.scan_and_index(root)?;
        Ok(())
    }
}

fn make_snippet(content: &str, q: &str) -> String {
    // Very lightweight snippet: find first occurrence of any term in q, else start of file
    let terms: Vec<String> = q.split_whitespace().map(|s| s.to_lowercase()).collect();
    let lc = content.to_lowercase();
    let mut idx = None;
    for t in &terms {
        if t.is_empty() { continue; }
        if let Some(i) = lc.find(t) { idx = Some(i); break; }
    }
    let start = idx.unwrap_or(0);
    let window_start = start.saturating_sub(60);
    let window_end = ((start + 200).min(content.len())).max(window_start);
    let mut snippet = content[window_start..window_end].to_string();
    snippet = snippet.replace('\n', " ").replace('\r', " ");
    if window_end < content.len() { snippet.push_str("..."); }
    snippet
}

fn should_skip(path: &Path) -> bool {
    let p = path.to_string_lossy();
    p.contains("/.git/")
        || p.contains("/target/")
    || p.ends_with("/target")
        || p.contains("/.backups/")
        || p.contains("/.import/")
        || p.contains("/.godot/")
    || p.contains("/.godot/imported/")
    || p.contains("/.godot/editor/")
        || p.contains("/.index_data/")
        || p.contains("/node_modules/")
    || p.contains("/docs/GODOT_ENGINE_DOCS/")
    || p.contains("/rust-book/")
}

/// Public helper to check whether a path should be skipped by the index.
pub fn is_skipped(path: &Path) -> bool { should_skip(path) }

//EOF