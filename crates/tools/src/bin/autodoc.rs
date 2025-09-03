use std::path::PathBuf;

fn main() {
    let root = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let report = tools::autodoc::ensure_autodocs(&root).expect("autodoc");
    println!("created: {} verified: {} skipped: {}", report.created.len(), report.verified.len(), report.skipped.len());
}