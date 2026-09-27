---cargo
[package]
edition = "2024"

[dependencies]
toml = "0.9"
---

//! Use the sibling TUIC checkout's real parsers without editing that checkout.
//! Run with `cargo +nightly -Zscript tests/config-editor/check-rust.rs [--offline]`.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
	match run() {
		Ok(()) => ExitCode::SUCCESS,
		Err(error) => {
			eprintln!("{error}");
			ExitCode::FAILURE
		}
	}
}

fn run() -> Result<(), Box<dyn Error>> {
	let offline = std::env::args().any(|arg| arg == "--offline");
	let root = repo_root();
	let tuic = root.parent().ok_or("repository root has no parent")?.join("tuic");
	if !tuic.is_dir() {
		return Err(format!("sibling TUIC checkout not found: {}", tuic.display()).into());
	}
	let target = root.join(".cache").join("config-editor-rust");
	std::fs::create_dir_all(&target)?;

	let manifest = std::fs::read_to_string(tuic.join("Cargo.toml"))?;
	let parsed: toml::Value = toml::from_str(&manifest)?;
	let patches = parsed
		.get("patch")
		.and_then(|patch| patch.get("crates-io"))
		.and_then(toml::Value::as_table)
		.ok_or("sibling TUIC Cargo.toml has no [patch.crates-io] table")?;

	let mut lines = vec![
		"[package]".to_owned(),
		"name = \"rust-proxy-docs-config-check\"".to_owned(),
		"version = \"0.0.0\"".to_owned(),
		"edition = \"2024\"".to_owned(),
		"[workspace]".to_owned(),
		"[[bin]]".to_owned(),
		"name = \"config-check\"".to_owned(),
		"path = \"main.rs\"".to_owned(),
		"[dependencies]".to_owned(),
	];
	for (name, path) in [("tuic-client", "crates/tuic-client"), ("tuic-server", "crates/tuic-server")] {
		lines.push(format!("{name} = {{ path = {} }}", quote_path(&tuic.join(path))));
	}
	lines.push("eyre = \"0.6\"".to_owned());
	lines.push("tokio = { version = \"1\", features = [\"full\"] }".to_owned());
	lines.push("rustls = { version = \"0.23\", features = [\"aws_lc_rs\"] }".to_owned());
	lines.push("[patch.crates-io]".to_owned());
	for (name, entry) in patches {
		let path = entry
			.get("path")
			.and_then(toml::Value::as_str)
			.ok_or_else(|| format!("[patch.crates-io].{name} has no path"))?;
		lines.push(format!("{name} = {{ path = {} }}", quote_path(&tuic.join(path))));
	}
	std::fs::write(target.join("Cargo.toml"), lines.join("\n") + "\n")?;
	std::fs::copy(script_dir().join("config-check.rs"), target.join("main.rs"))?;
	// Seed with TUIC's lock so Git revisions and existing versions match the baseline.
	std::fs::copy(tuic.join("Cargo.lock"), target.join("Cargo.lock"))?;

	let fixtures = root.join(".cache").join("config-editor-fixtures");
	cargo(&root)
		.args(["run", "--locked", "--example", "fixtures", "--"])
		.arg(&fixtures)
		.status_checked("fixture generation")?;

	let mut command = cargo(&tuic);
	command.args(["run", "--manifest-path"]).arg(target.join("Cargo.toml"));
	if offline {
		command.arg("--offline");
	}
	command.arg("--").arg(&fixtures).status_checked("real TUIC parser check")?;
	Ok(())
}

fn cargo(dir: &Path) -> Command {
	let mut command = Command::new("cargo");
	command.current_dir(dir);
	// `cargo +nightly -Zscript` would otherwise leak its toolchain into child builds.
	command.env_remove("RUSTUP_TOOLCHAIN");
	command
}

trait StatusChecked {
	fn status_checked(&mut self, label: &str) -> Result<(), Box<dyn Error>>;
}

impl StatusChecked for Command {
	fn status_checked(&mut self, label: &str) -> Result<(), Box<dyn Error>> {
		if self.status()?.success() {
			Ok(())
		} else {
			Err(format!("{label} failed").into())
		}
	}
}

fn quote_path(path: &Path) -> String {
	format!("{:?}", path.to_string_lossy())
}

fn script_dir() -> PathBuf {
	std::env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."))
}

fn repo_root() -> PathBuf {
	script_dir().join("..").join("..").canonicalize().unwrap_or_else(|_| PathBuf::from("."))
}
