use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "devspace-sweeper")] 
#[command(about = "Find and safely clean dev junk across projects", long_about = None)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
	/// Scan for recognizable build artifacts and caches
	Scan {
		/// Root path to scan (defaults to current directory)
		#[arg(short, long)]
		path: Option<PathBuf>,
		/// Custom recipes YAML path
		#[arg(short = 'r', long = "recipes")]
		recipes: Option<PathBuf>,
		/// Max directory depth to traverse
		#[arg(long, default_value_t = 8)]
		max_depth: usize,
	},
	/// Suggest safe cleanup candidates and .gitignore hints
	Suggest {
		#[arg(short, long)]
		path: Option<PathBuf>,
		#[arg(short = 'r', long = "recipes")]
		recipes: Option<PathBuf>,
	},
	/// Clean selected artifacts (dry-run by default)
	Clean {
		#[arg(short, long)]
		path: Option<PathBuf>,
		#[arg(short = 'r', long = "recipes")]
		recipes: Option<PathBuf>,
		/// Actually delete files (omit for dry-run)
		#[arg(long, default_value_t = false)]
		dry_run: bool,
		/// Only delete items older than N days
		#[arg(long)]
		max_age_days: Option<u64>,
		/// Keep items modified within the last N days
		#[arg(long)]
		keep_recent_days: Option<u64>,
	},
	/// Generate a Markdown report summarizing findings
	Report {
		#[arg(short, long)]
		path: Option<PathBuf>,
		#[arg(short = 'r', long = "recipes")]
		recipes: Option<PathBuf>,
		/// Output file path (defaults to report.md in cwd)
		#[arg(short, long)]
		output: Option<PathBuf>,
	},
	/// Append suggested patterns to .gitignore at the root
	GenIgnore {
		#[arg(short, long)]
		path: Option<PathBuf>,
		#[arg(short = 'r', long = "recipes")]
		recipes: Option<PathBuf>,
		/// Preview changes without writing
		#[arg(long, default_value_t = false)]
		dry_run: bool,
	},
}
