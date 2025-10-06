mod commands;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "zen", about = "Keep your repos at peace 🧘‍♂️", author, version, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Delete all node_modules folders recursively
    Sweep,
    /// Delete local branches that no longer exist on origin
    Prune,
    /// Show contributors ranked by number of commits
    Pulse {
        /// Rank by lines changed instead of commit count
        #[arg(short, long)]
        lines: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sweep => commands::sweep::run()?,
        Commands::Prune => commands::prune::run()?,
        Commands::Pulse { lines } => commands::pulse::run(lines)?,
    }

    Ok(())
}