use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "godot-analyzer", version, about = "Analyze a Godot project for configuration and addon health", long_about = None)]
struct Args {
    #[arg(short, long)]
    root: Option<std::path::PathBuf>,
    #[arg(long)]
    json: bool,
}

fn main() {
    let args = Args::parse();
    let root = args.root.unwrap_or_else(|| std::env::current_dir().unwrap());
    let report = godot_analyzer::analyze_project(&root).expect("analyze");
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!("Godot project at {}", root.display());
        println!("project_format_version: {:?}", report.project_format_version);
        println!("addons: {}", if report.addons.is_empty() { "none".into() } else { report.addons.join(", ") });
        println!("export presets: {}", if report.export_presets.is_empty() { "none".into() } else { report.export_presets.iter().map(|p| format!("{} ({})", p.name, p.platform)).collect::<Vec<_>>().join(", ") });
        println!("issues: {}", report.issues.len());
    }
}
