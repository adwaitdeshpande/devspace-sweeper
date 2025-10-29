mod cli;
mod recipes;
mod scan;
mod clean;
mod report;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
	let cli = Cli::parse();
	match cli.command {
		Commands::Scan { path, recipes, max_depth } => scan::run_scan(path, recipes, max_depth)?,
		Commands::Suggest { path, recipes } => scan::run_suggest(path, recipes)?,
		Commands::Clean { path, recipes, dry_run, max_age_days, keep_recent_days } => {
			clean::run_clean(path, recipes, dry_run, max_age_days, keep_recent_days)?
		}
		Commands::Report { path, recipes, output } => report::run_report(path, recipes, output)?,
		Commands::GenIgnore { path, recipes, dry_run } => scan::run_gen_ignore(path, recipes, dry_run)?,
	}
	Ok(())
}
