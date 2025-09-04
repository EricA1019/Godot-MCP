use clap::Parser;
use std::path::PathBuf;
use godot_analyzer::{
    analyze_project, scene_issues_as_report_with, signal_graph_dot, signal_issues_as_report, structure_fix, GodotProjectReport, SceneCheckOptions, Severity, to_junit, to_sarif,
};

#[derive(Parser, Debug)]
#[command(name = "godot-analyzer", version, about = "Analyze a Godot project for configuration and addon health", long_about = None)]
struct Args {
    #[arg(short, long)]
    root: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    /// Minimum severity to include in outputs (info|warn|error)
    #[arg(long)]
    min_severity: Option<String>,
    /// Write SARIF output to this file
    #[arg(long)]
    sarif_out: Option<PathBuf>,
    /// Write JUnit XML output to this file
    #[arg(long)]
    junit_out: Option<PathBuf>,
    /// Exit with code 2 if any issue meets or exceeds this severity (info|warn|error)
    #[arg(long)]
    fail_on: Option<String>,
    /// Validate scenes (.tscn) and include findings in outputs
    #[arg(long)]
    validate_scenes: bool,
    /// Validate signal connections in scenes and include findings in outputs
    #[arg(long)]
    validate_signals: bool,
    /// Optionally write scene findings as a standalone JSON file
    #[arg(long)]
    scene_json_out: Option<PathBuf>,
    /// Select which scene checks to run (repeatable). Options: script,properties,subresource,preload,load.
    #[arg(long = "scene-check")] 
    scene_checks: Vec<String>,
    /// Optionally write a DOT graph of signal connections across scenes
    #[arg(long)]
    signal_dot_out: Option<PathBuf>,
    /// Structure fix: plan (dry-run) only; prints JSON to stdout unless --json-out provided
    #[arg(long)]
    structure_fix: bool,
    /// Write structure fix plan JSON to this file instead of stdout
    #[arg(long)]
    structure_fix_json_out: Option<PathBuf>,
    /// Apply structure fix (implies --structure_fix). Prints JSON summary.
    #[arg(long)]
    structure_fix_apply: bool,
}

fn main() {
    let args = Args::parse();
    let root = args.root.unwrap_or_else(|| std::env::current_dir().unwrap());
    let mut report = analyze_project(&root).expect("analyze");

    // Structure fix planning/apply
    if args.structure_fix || args.structure_fix_apply {
        let plan = structure_fix::plan_structure_fix(&root);
        if args.structure_fix_apply {
            let sum = structure_fix::apply_structure_fix(&root, &plan).expect("apply structure fix");
            println!("{}", serde_json::to_string_pretty(&sum).unwrap());
        } else {
            let s = serde_json::to_string_pretty(&plan).unwrap();
            if let Some(p) = args.structure_fix_json_out.as_ref() {
                std::fs::write(p, s).expect("write structure fix json");
            } else {
                println!("{}", s);
            }
        }
        return;
    }

    if args.validate_scenes {
        let mut opts = SceneCheckOptions::default();
        if !args.scene_checks.is_empty() {
            // Disable all, then enable selected
            opts = SceneCheckOptions { script: false, properties: false, subresource: false, preload: false, load: false };
            for c in &args.scene_checks {
                match c.as_str() {
                    "script" => opts.script = true,
                    "properties" => opts.properties = true,
                    "subresource" => opts.subresource = true,
                    "preload" => opts.preload = true,
                    "load" => opts.load = true,
                    _ => {}
                }
            }
        }
        let scene_issues = scene_issues_as_report_with(&root, &opts);
        if let Some(p) = args.scene_json_out.as_ref() {
            std::fs::write(p, serde_json::to_vec_pretty(&scene_issues).unwrap()).expect("write scene json");
        }
    report.issues.extend(scene_issues);
    // Keep deterministic ordering after merge
    report.issues.sort_by(|a, b| a.severity.cmp(&b.severity).then(a.message.cmp(&b.message)));
    }

    if args.validate_signals {
        let sig_issues = signal_issues_as_report(&root);
        report.issues.extend(sig_issues);
        report.issues.sort_by(|a, b| a.severity.cmp(&b.severity).then(a.message.cmp(&b.message)));
    }

    // Optional DOT graph export for signals
    if let Some(p) = args.signal_dot_out.as_ref() {
        let dot = signal_graph_dot(&root);
        std::fs::write(p, dot).expect("write signal dot");
    }

    // Optional filtering by minimum severity for outputs
    let mut filtered: Option<GodotProjectReport> = None;
    if let Some(ms) = args.min_severity.as_deref().and_then(parse_severity) {
        let mut r = report.clone();
        r.issues.retain(|i| i.severity >= ms);
        filtered = Some(r);
    }
    let out_ref = filtered.as_ref().unwrap_or(&report);

    if args.json {
        println!("{}", serde_json::to_string_pretty(out_ref).unwrap());
    } else {
        println!("Godot project at {}", root.display());
        println!("project_format_version: {:?}", out_ref.project_format_version);
        println!("addons: {}", if out_ref.addons.is_empty() { "none".into() } else { out_ref.addons.join(", ") });
        println!("export presets: {}", if out_ref.export_presets.is_empty() { "none".into() } else { out_ref.export_presets.iter().map(|p| format!("{} ({})", p.name, p.platform)).collect::<Vec<_>>().join(", ") });
        println!("issues: {}", out_ref.issues.len());
    }

    if let Some(p) = args.sarif_out.as_ref() {
        let v = to_sarif(out_ref);
        std::fs::write(p, serde_json::to_vec_pretty(&v).unwrap()).expect("write sarif");
    }
    if let Some(p) = args.junit_out.as_ref() {
        let s = to_junit(out_ref);
        std::fs::write(p, s).expect("write junit");
    }

    if let Some(th) = args.fail_on.as_deref().and_then(parse_severity) {
        if report.issues.iter().any(|i| i.severity >= th) {
            std::process::exit(2);
        }
    }
}

fn parse_severity(s: &str) -> Option<Severity> {
    match s.to_lowercase().as_str() {
        "info" => Some(Severity::Info),
        "warn" | "warning" => Some(Severity::Warn),
        "error" | "err" => Some(Severity::Error),
        _ => None,
    }
}
