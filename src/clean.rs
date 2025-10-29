use crate::recipes::RecipeFile;
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use humansize::{format_size, DECIMAL};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn run_clean(path: Option<PathBuf>, recipes_path: Option<PathBuf>, dry_run: bool, max_age_days: Option<u64>, keep_recent_days: Option<u64>) -> Result<()> {
	let root = path.unwrap_or(std::env::current_dir()?);
	let recipes = RecipeFile::load(recipes_path)?;
	let globset = recipes.compile_globset()?;
	let max_age = max_age_days.map(|d| Duration::days(d as i64));
	let keep_recent = keep_recent_days.map(|d| Duration::days(d as i64));

	let mut bytes = 0u64;
	let mut items = 0usize;

	for entry in WalkDir::new(&root).follow_links(false).into_iter().filter_map(|e| e.ok()) {
		let path = entry.path();
		let rel = path.strip_prefix(&root).unwrap_or(path);
		if !globset.is_match(rel.to_string_lossy().as_ref()) { continue; }

		if !passes_age_filters(path, max_age, keep_recent)? { continue; }

		let b = size_hint(path)?;
		if dry_run {
			println!("Would delete {} ({}).", rel.display(), format_size(b, DECIMAL));
		} else {
			delete_path(path).with_context(|| format!("Failed to delete {}", path.display()))?;
			println!("Deleted {} ({}).", rel.display(), format_size(b, DECIMAL));
		}
		bytes = bytes.saturating_add(b);
		items += 1;
	}

	if dry_run {
		println!("Dry run complete. Would reclaim {} across {} items.", format_size(bytes, DECIMAL), items);
	} else {
		println!("Cleanup complete. Reclaimed {} across {} items.", format_size(bytes, DECIMAL), items);
	}
	Ok(())
}

fn size_hint(path: &Path) -> Result<u64> {
	let md = fs::symlink_metadata(path)?;
	if md.is_file() { return Ok(md.len()); }
	let mut total = 0u64;
	for e in WalkDir::new(path).follow_links(false).into_iter().filter_map(|e| e.ok()) {
		let md = match e.metadata() { Ok(m) => m, Err(_) => continue };
		if md.is_file() { total = total.saturating_add(md.len()); }
	}
	Ok(total)
}

fn delete_path(path: &Path) -> Result<()> {
	if path.is_file() { fs::remove_file(path)?; }
	else { fs::remove_dir_all(path)?; }
	Ok(())
}

fn passes_age_filters(path: &Path, max_age: Option<Duration>, keep_recent: Option<Duration>) -> Result<bool> {
	let md = match fs::symlink_metadata(path) { Ok(m) => m, Err(_) => return Ok(false) };
	let modified = match md.modified() { Ok(m) => m, Err(_) => return Ok(true) };
	let modified = chrono::DateTime::<Utc>::from(modified);
	let now = Utc::now();
	if let Some(k) = keep_recent { if now - modified < k { return Ok(false); } }
	if let Some(m) = max_age { if now - modified > m { return Ok(true); } else { return Ok(false); } }
	Ok(true)
}
