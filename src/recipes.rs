use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeFile {
	pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
	pub name: String,
	pub description: Option<String>,
	pub globs: Vec<String>,
	#[serde(default = "default_true")]
	pub safe_delete: bool,
	#[serde(default)]
	pub max_age_days: Option<u64>,
}

fn default_true() -> bool { true }

impl RecipeFile {
	pub fn load(path: Option<PathBuf>) -> Result<Self> {
		let path = match path {
			Some(p) => p,
			None => default_recipes_path(),
		};
		let content = fs::read_to_string(&path)
			.with_context(|| format!("Failed to read recipes at {}", path.display()))?;
		let rf: RecipeFile = serde_yaml::from_str(&content)
			.with_context(|| format!("Failed to parse YAML recipes at {}", path.display()))?;
		Ok(rf)
	}

	pub fn compile_globset(&self) -> Result<GlobSet> {
		let mut builder = GlobSetBuilder::new();
		for rule in &self.rules {
			for g in &rule.globs {
				builder.add(Glob::new(g)?);
			}
		}
		Ok(builder.build()?)
	}
}

pub fn default_recipes_path() -> PathBuf {
	let local = Path::new("recipes/default.yml");
	if local.exists() { return local.to_path_buf(); }
	// Fallback to embedded default (not implemented yet)
	local.to_path_buf()
}
