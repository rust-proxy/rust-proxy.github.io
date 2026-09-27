---cargo
[package]
edition = "2024"

[dependencies]
serde_json = "1"
serde_yaml = "0.9"
toml = "0.9"
---

//! Independent TOML/JSON/YAML parsers for the Rust-generated fixture data.
//! Run with `cargo +nightly -Zscript tests/config-editor/roundtrip.rs [file]`;
//! reads standard input when no path is given.

use std::error::Error;
use std::io::Read;

fn main() -> Result<(), Box<dyn Error>> {
	let text = match std::env::args().nth(1) {
		Some(path) => std::fs::read_to_string(path)?,
		None => {
			let mut buffer = String::new();
			std::io::stdin().read_to_string(&mut buffer)?;
			buffer
		}
	};
	let cases: Vec<serde_json::Value> = serde_json::from_str(&text)?;
	for case in &cases {
		let name = case["name"].as_str().unwrap_or("<unnamed>");
		let expected = &case["expected"];
		let formats = &case["formats"];
		for (format, parse) in [
			("toml", parse_toml as fn(&str) -> Result<serde_json::Value, String>),
			("json", parse_json),
			("yaml", parse_yaml),
		] {
			let source = formats[format]
				.as_str()
				.ok_or_else(|| format!("missing {format} for {name}"))?;
			let parsed = parse(source).map_err(|error| format!("{name} / {format}: {error}"))?;
			if &parsed != expected {
				// Never include generated credentials in diagnostics.
				return Err(format!("Round-trip mismatch: {name} / {format}").into());
			}
		}
	}
	println!("{} objects round-tripped across TOML, JSON and YAML", cases.len());
	Ok(())
}

fn parse_toml(source: &str) -> Result<serde_json::Value, String> {
	toml::from_str(source).map_err(|error| error.to_string())
}

fn parse_json(source: &str) -> Result<serde_json::Value, String> {
	serde_json::from_str(source).map_err(|error| error.to_string())
}

fn parse_yaml(source: &str) -> Result<serde_json::Value, String> {
	serde_yaml::from_str(source).map_err(|error| error.to_string())
}
