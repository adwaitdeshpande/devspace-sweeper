use crate::recipes::{RecipeFile};
use anyhow::{Context, Result};
use humansize::{format_size, DECIMAL};
use ignore::WalkBuilder;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MatchInfo {
	pub path: PathBuf,
	pub bytes: u64,
}

#[derive(Debug, Default, Clone)]
pub struct RuleAggregate {
	pub total_bytes: u64,
	pub count: usize,
	pub examples: Vec<PathBuf>,
}

pub fn run_scan(path: Option<PathBuf>, recipes_path: Option<PathBuf>, max_depth: usize) -> Result<()> {
	let root = path.unwrap_or(std::env::current_dir()?);
	let recipes = RecipeFile::load(recipes_path)?;
	let globset = recipes.compile_globset()?;

	let (agg, total) = scan_dir(&root, &globset, max_depth)?;
	println!("Scanning {}", root.display());
	println!("Found total {} across {} matched groups", format_size(total, DECIMAL), agg.len());
	for (pattern_group, summary) in agg {
		println!("- {}: {} in {} items", pattern_group, format_size(summary.total_bytes, DECIMAL), summary.count);
		for ex in summary.examples.iter().take(3) {
			println!("  example: {}", ex.display());
		}
	}
	Ok(())
}

pub fn run_suggest(path: Option<PathBuf>, recipes_path: Option<PathBuf>) -> Result<()> {
	let root = path.unwrap_or(std::env::current_dir()?);
	let recipes = RecipeFile::load(recipes_path)?;
	let globset = recipes.compile_globset()?;
	let (agg, _total) = scan_dir(&root, &globset, 8)?;

	println!("Suggestions for {}", root.display());
	let mut ranked: Vec<(String, RuleAggregate)> = agg.into_iter().collect();
	ranked.sort_by_key(|(_, v)| std::cmp::Reverse(v.total_bytes));

	let mut ignore_hints: HashMap<String, usize> = HashMap::new();
	for (group, summary) in &ranked {
		let safety = safety_score_for_group(group);
		println!("- {} -> {} ({} items) | safety: {}%", group, format_size(summary.total_bytes, DECIMAL), summary.count, safety);
		let hint = gitignore_hint_for_group(group);
		if let Some(h) = hint { *ignore_hints.entry(h).or_insert(0) += 1; }
	}

	if !ignore_hints.is_empty() {
		println!("\n.gitignore hints (add to your repo root):");
		let mut hints: Vec<(String, usize)> = ignore_hints.into_iter().collect();
		hints.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
		for (h, c) in hints.iter().take(8) {
			println!("  {}  # seen in {} places", h, c);
		}
	}
	println!("\nHint: review safety score; items like node_modules, dist, target are safe to delete.");
	Ok(())
}

pub fn run_gen_ignore(path: Option<PathBuf>, recipes_path: Option<PathBuf>, dry_run: bool) -> Result<()> {
	let root = path.unwrap_or(std::env::current_dir()?);
	let recipes = RecipeFile::load(recipes_path)?;
	let globset = recipes.compile_globset()?;
	let (agg, _total) = scan_dir(&root, &globset, 8)?;

	let mut ranked: Vec<(String, RuleAggregate)> = agg.into_iter().collect();
	ranked.sort_by_key(|(_, v)| std::cmp::Reverse(v.total_bytes));
	let mut hints: Vec<String> = Vec::new();
	for (group, _summary) in &ranked {
		if let Some(h) = gitignore_hint_for_group(group) {
			if !hints.contains(&h) { hints.push(h); }
		}
	}
	if hints.is_empty() {
		println!("No .gitignore hints found.");
		return Ok(());
	}
	let gi_path = root.join(".gitignore");
	if dry_run {
		println!("Would append to {}:\n", gi_path.display());
		for h in &hints { println!("{}", h); }
		return Ok(());
	}
	let mut file = if gi_path.exists() {
		fs::OpenOptions::new().append(true).open(&gi_path)
			.with_context(|| format!("Failed to open {}", gi_path.display()))?
	} else {
		fs::OpenOptions::new().create(true).write(true).open(&gi_path)
			.with_context(|| format!("Failed to create {}", gi_path.display()))?
	};
	writeln!(file, "\n# Added by devspace-sweeper")?;
	for h in &hints { writeln!(file, "{}", h)?; }
	println!("Appended {} patterns to {}", hints.len(), gi_path.display());
	Ok(())
}

fn safety_score_for_group(group: &str) -> u8 {
	match group {
		"node_modules" | "dist" | "build" | "__pycache__" | ".pytest_cache" | "target" | "DerivedData" | ".parcel-cache" | ".vite" | ".next" | ".nuxt" => 95,
		"coverage" | ".nyc_output" => 90,
		".venv" | "venv" | "env" => 50,
		_ => 70,
	}
}

fn gitignore_hint_for_group(group: &str) -> Option<String> {
	match group {
		"node_modules" => Some(String::from("node_modules/")),
		"dist" => Some(String::from("dist/")),
		"build" => Some(String::from("build/")),
		"__pycache__" => Some(String::from("**/__pycache__/")),
		".pytest_cache" => Some(String::from(".pytest_cache/")),
		"target" => Some(String::from("target/")),
		"DerivedData" => Some(String::from("DerivedData/")),
		"coverage" => Some(String::from("coverage/")),
		".nyc_output" => Some(String::from(".nyc_output/")),
		".parcel-cache" => Some(String::from(".parcel-cache/")),
		".vite" => Some(String::from(".vite/")),
		".next" => Some(String::from(".next/")),
		".nuxt" => Some(String::from(".nuxt/")),
		_ => None,
	}
}

fn scan_dir(root: &Path, globset: &globset::GlobSet, max_depth: usize) -> Result<(BTreeMap<String, RuleAggregate>, u64)> {
	let mut totals: BTreeMap<String, RuleAggregate> = BTreeMap::new();
	let mut seen_dirs: HashSet<PathBuf> = HashSet::new();
	let mut total_bytes: u64 = 0;

	let mut builder = WalkBuilder::new(root);
	builder.git_ignore(true).git_exclude(true).hidden(false).max_depth(Some(max_depth));

	for result in builder.build() {
		let dent = match result { Ok(d) => d, Err(_) => continue };
		let path = dent.path();
		let rel = path.strip_prefix(root).unwrap_or(path);
		if rel.as_os_str().is_empty() { continue; }
		let rel_str = rel.to_string_lossy();
		if !globset.is_match(rel_str.as_ref()) { continue; }
		let md = match fs::symlink_metadata(path) { Ok(m) => m, Err(_) => continue };
		let mut size = 0u64;
		if md.is_file() {
			size = md.len();
		} else if md.is_dir() {
			if seen_dirs.contains(path) { continue; }
			seen_dirs.insert(path.to_path_buf());
			size = dir_size(path)?;
		}
		let group_key = group_from_path(rel);
		let entry = totals.entry(group_key).or_default();
		entry.total_bytes = entry.total_bytes.saturating_add(size);
		entry.count += 1;
		if entry.examples.len() < 5 { entry.examples.push(rel.to_path_buf()); }
		total_bytes = total_bytes.saturating_add(size);
	}
	Ok((totals, total_bytes))
}

fn group_from_path(path: &Path) -> String {
	let comps: Vec<_> = path.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
	if comps.is_empty() { return String::from("root"); }
	// Group by the last component (e.g., node_modules, target, build)
	comps.last().cloned().unwrap_or_else(|| String::from("misc"))
}

fn dir_size(path: &Path) -> Result<u64> {
	let mut total = 0u64;
	let mut builder = WalkBuilder::new(path);
	builder.git_ignore(true).git_exclude(true).hidden(false);
	for result in builder.build() {
		let dent = match result { Ok(d) => d, Err(_) => continue };
		let md = match dent.metadata() { Ok(m) => m, Err(_) => continue };
		if md.is_file() { total = total.saturating_add(md.len()); }
	}
	Ok(total)
}
