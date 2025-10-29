use crate::recipes::RecipeFile;
use anyhow::Result;
use humansize::{format_size, DECIMAL};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn run_report(path: Option<PathBuf>, recipes_path: Option<PathBuf>, output: Option<PathBuf>) -> Result<()> {
	let root = path.unwrap_or(std::env::current_dir()?);
	let recipes = RecipeFile::load(recipes_path)?;
	let globset = recipes.compile_globset()?;
	let out = output.unwrap_or_else(|| PathBuf::from("report.md"));

	let mut lines: Vec<String> = Vec::new();
	lines.push(format!("# DevSpace Sweeper Report"));
	lines.push(format!("Root: {}\n", root.display()));
	let mut total = 0u64;

	let mut builder = WalkBuilder::new(&root);
	builder.git_ignore(true).git_exclude(true).hidden(false);
	for result in builder.build() {
		let dent = match result { Ok(d) => d, Err(_) => continue };
		let path = dent.path();
		let rel = path.strip_prefix(&root).unwrap_or(path);
		if !globset.is_match(rel.to_string_lossy().as_ref()) { continue; }
		let size = match dent.metadata() { Ok(m) => if m.is_file() { m.len() } else { 0 }, Err(_) => 0 };
		if size > 0 { total = total.saturating_add(size); }
		lines.push(format!("- {} ({})", rel.display(), format_size(size, DECIMAL)));
	}
	lines.push(format!("\nTotal identifiable size: {}", format_size(total, DECIMAL)));
	fs::write(&out, lines.join("\n"))?;
	println!("Report written to {}", out.display());
	Ok(())
}

pub fn compute_ignore_hints(root: &PathBuf, patterns: &[&str]) -> HashMap<String, usize> {
	let mut hints: HashMap<String, usize> = HashMap::new();
	for p in patterns {
		*hints.entry((*p).to_string()).or_insert(0) += 1;
	}
	hints
}
