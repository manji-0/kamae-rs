use std::path::PathBuf;

use clap::Parser;
use kamae_review_probe::output::{render_json, render_text};
use kamae_review_probe::probe::probe_paths;

#[derive(Parser)]
#[command(
    name = "review-probe",
    about = "Collect kamae-rs review leads from Rust files using AST analysis."
)]
struct Args {
    #[arg(help = "Files or directories to scan", default_value = ".")]
    paths: Vec<PathBuf>,

    #[arg(long, help = "Emit JSON instead of Markdown text")]
    json: bool,

    #[arg(long, default_value_t = 80, help = "Maximum leads per text section")]
    limit: usize,
}

fn main() {
    let args = Args::parse();
    let root = std::env::current_dir().expect("current directory");
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths
    };

    let output = probe_paths(&root, &paths);
    if args.json {
        println!("{}", render_json(&output));
    } else {
        println!("{}", render_text(&output, args.limit));
    }
}
