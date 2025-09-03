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

    /// Minimum severity to include (info, warn, error)
    #[arg(long, value_name = "LEVEL", default_value = "info")]
    min_severity: String,
}

fn main() {
    let args = Args::parse();
    let root = args.root.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let mut report = tools::metatagger::run(&root).expect("metatagger");

    let min = match args.min_severity.as_str() {
        "error" => tools::metatagger::Severity::Error,
        "warn" => tools::metatagger::Severity::Warn,
        _ => tools::metatagger::Severity::Info,
    };
    report.findings.retain(|f| f.severity >= min);

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
