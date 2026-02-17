mod config;
mod renamer;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use config::Config;

#[derive(Parser)]
#[command(name = "docs-manager-cli", version, about = format!("docs-manager-cli v{} — Manage doc files", env!("CARGO_PKG_VERSION")))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Rename docs to yyyy-mm-dd-hh-mm-ss-<title>.ext format
    Rename {
        /// Docs directory path
        #[arg(short, long)]
        dir: Option<String>,

        /// Actually perform renames (default: dry-run)
        #[arg(short = 'x', long)]
        execute: bool,

        /// Config file path
        #[arg(short, long, default_value = ".doc-manager-cli/config.toml")]
        config: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Rename {
            dir,
            execute,
            config: config_path,
        } => {
            let cfg = Config::load(&config_path, dir.as_deref());
            let ops = renamer::plan_renames(&cfg);

            if ops.is_empty() {
                println!("nothing to rename");
                return;
            }

            if execute {
                let count = renamer::execute_renames(&ops);
                println!("{count} file(s) renamed");
            } else {
                println!("dry-run (use --execute or -x to apply):\n");
                for op in &ops {
                    println!("  {} -> {}", op.from.display(), op.to.display());
                }
                println!("\n{} file(s) would be renamed", ops.len());
            }
        }
    }
}
