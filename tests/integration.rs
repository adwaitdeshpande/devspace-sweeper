use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_fixture() -> (TempDir, PathBuf) {
	let td = TempDir::new().expect("tempdir");
	let root = td.path().to_path_buf();
	// Create common junk
	fs::create_dir_all(root.join("proj/node_modules/pkg" )).unwrap();
	fs::create_dir_all(root.join("proj/target/debug" )).unwrap();
	fs::create_dir_all(root.join("proj/__pycache__" )).unwrap();
	fs::write(root.join("proj/node_modules/pkg/index.js"), b"console.log('x');").unwrap();
	fs::write(root.join("proj/target/debug/app"), b"bin").unwrap();
	fs::write(root.join("proj/__pycache__/a.pyc"), b"pyc").unwrap();
	(td, root)
}

#[test]
fn scan_should_list_groups() {
	let (_td, root) = create_fixture();
	let mut cmd = Command::cargo_bin("devspace-sweeper").unwrap();
	cmd.arg("scan").arg("--path").arg(root);
	cmd.assert()
		.success()
		.stdout(predicate::str::contains("node_modules"))
		.stdout(predicate::str::contains("target"))
		.stdout(predicate::str::contains("__pycache__"));
}

#[test]
fn suggest_should_print_hints_and_safety() {
	let (_td, root) = create_fixture();
	let mut cmd = Command::cargo_bin("devspace-sweeper").unwrap();
	cmd.arg("suggest").arg("--path").arg(root);
	cmd.assert()
		.success()
		.stdout(predicate::str::contains("safety:"))
		.stdout(predicate::str::contains(".gitignore hints"));
}

#[test]
fn gen_ignore_dry_run_should_output_patterns() {
	let (_td, root) = create_fixture();
	let mut cmd = Command::cargo_bin("devspace-sweeper").unwrap();
	cmd.arg("gen-ignore").arg("--path").arg(&root).arg("--dry-run").arg("true");
	cmd.assert()
		.success()
		.stdout(predicate::str::contains("node_modules/"))
		.stdout(predicate::str::contains("target/"))
		.stdout(predicate::str::contains("**/__pycache__/"));
}
