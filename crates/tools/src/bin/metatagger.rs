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

    /// Emit SARIF to the given file path
    #[arg(long, value_name = "FILE")]
    sarif_out: Option<PathBuf>,

    /// Emit JUnit XML to the given file path
    #[arg(long, value_name = "FILE")]
    junit_out: Option<PathBuf>,

    /// Fail the process if any finding has severity >= LEVEL
    #[arg(long, value_name = "LEVEL")]
    fail_on: Option<String>,
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

    // Optional outputs
    if let Some(p) = &args.sarif_out { std::fs::write(p, serde_json::to_string_pretty(&tools::metatagger::to_sarif(&report)).unwrap()).expect("write sarif"); }
    if let Some(p) = &args.junit_out { std::fs::write(p, tools::metatagger::to_junit(&report)).expect("write junit"); }

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

    // CI gating
    if let Some(level) = args.fail_on.as_deref() {
        let gate = match level { "error" => tools::metatagger::Severity::Error, "warn" => tools::metatagger::Severity::Warn, _ => tools::metatagger::Severity::Info };
        if report.findings.iter().any(|f| f.severity >= gate) {
            std::process::exit(2);
        }
    }
}
