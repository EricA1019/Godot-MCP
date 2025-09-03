use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "autodoc", version, about = "Ensure CTS docs exist and update managed regions", long_about = None)]
struct Args {
    /// Workspace root to operate on
    #[arg(short, long, value_name = "PATH")]
    root: Option<PathBuf>,

    /// Dry run: report changes without writing
    #[arg(long)]
    dry_run: bool,

    /// Check only: exit non-zero if changes would be made
    #[arg(long)]
    check: bool,

    /// Output JSON report
    #[arg(long)]
    json: bool,
}

fn main() {
    let args = Args::parse();
    let root = args.root.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let opts = tools::autodoc::EnsureOpts { dry_run: args.dry_run, check_only: args.check };
    let report = tools::autodoc::ensure_autodocs_opts(&root, opts).expect("autodoc");

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!(
            "created: {} updated: {} verified: {} skipped: {}",
            report.created.len(),
            report.updated.len(),
            report.verified.len(),
            report.skipped.len()
        );
    }

    if args.check && (!report.created.is_empty() || !report.updated.is_empty()) {
        std::process::exit(2);
    }
}