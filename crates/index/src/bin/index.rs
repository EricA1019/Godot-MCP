// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ Binary: index-cli                                                   ┃
// ┃ Purpose: Scan/query the Master Index (full-text)                    ┃
// ┃ Author: EricA1019                                                   ┃
// ┃ Last Updated: 2025-09-02                                           ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use anyhow::{bail, Result};
use index::{IndexPaths, SearchIndex};
use std::path::{Path, PathBuf};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logs() {
    let env = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry().with(env).with(tracing_subscriber::fmt::layer()).init();
}

fn main() -> Result<()> {
    init_logs();
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() { print_help(); bail!("no args"); }
    let cmd = args.remove(0);

    let root = PathBuf::from(".");
    let data_dir = PathBuf::from(".index_data");
    let paths = IndexPaths { root: root.clone(), data_dir };
    let mut idx = SearchIndex::open(&paths)?;

    match cmd.as_str() {
        "scan" => {
            let n = idx.scan_and_index(&root)?;
            println!("Indexed {} files", n);
        }
        "query" => {
            let q = args.join(" ");
            let hits = idx.query(&q, 10)?;
            for (score, path) in hits { println!("{score:.3}\t{path}"); }
        }
        "watch" => {
            let dir = args.get(0).cloned().unwrap_or_else(|| String::from("."));
            idx.scan_and_index(Path::new(&dir))?;
            println!("Initial scan complete. Watching for changes...");
            idx.watch(Path::new(&dir))?;
        }
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!("Usage: index-cli scan|query <terms...>|watch [dir]");
}

//EOF