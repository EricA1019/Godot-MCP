use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "metatagger", version, about = "Classify repo and update PROJECT_INDEX cleanup section", long_about = None)]
struct Args {
    /// Workspace root to operate on
    #[arg(short, long, value_name = "PATH")]
    root: Option<PathBuf>,

    /// Output JSON report
    #[arg(long)]
    json: bool,
}

fn main() {
    let args = Args::parse();
    let root = args.root.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let report = tools::metatagger::run(&root).expect("metatagger");

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!(
            "findings: {}{}",
            report.findings.len(),
            report
                .updated
                .as_ref()
                .map(|p| format!("; updated {}", p.display()))
                .unwrap_or_default()
        );
    }
}
