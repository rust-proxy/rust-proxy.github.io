---cargo
[package]
edition = "2024"
---

//! Check assembled site links, fragments, and the standalone editor's assets
//! and privacy. Run with `cargo +nightly -Zscript tests/config-editor/check-site.rs`
//! after building `site/`.

use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

#[derive(Default)]
struct Page {
	links: Vec<String>,
	assets: Vec<String>,
	ids: BTreeSet<String>,
}

struct Entry {
	canonical: PathBuf,
	relative: String,
	page: Page,
}

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
	let site = repo_root().join("site");
	if !site.is_dir() {
		return Err(format!("assembled site not found: {}; run `just build` first", site.display()).into());
	}

	let mut entries = Vec::new();
	collect(&site, &site, &mut entries)?;

	let mut by_canonical: HashMap<PathBuf, usize> = HashMap::new();
	for (index, entry) in entries.iter().enumerate() {
		by_canonical.insert(entry.canonical.clone(), index);
	}

	let mut errors = Vec::new();
	for entry in &entries {
		for href in &entry.page.links {
			check_link(href, &entry.canonical, &site, &entries, &by_canonical, &entry.relative, &mut errors);
		}
	}

	check_editor(&site, &mut errors)?;

	if !errors.is_empty() {
		return Err(errors.join("\n").into());
	}
	println!(
		"All local links and fragments passed across {} pages; standalone WASM assets present and analytics absent",
		entries.len()
	);
	Ok(())
}

fn check_link(
	href: &str,
	page_path: &Path,
	site: &Path,
	entries: &[Entry],
	by_canonical: &HashMap<PathBuf, usize>,
	relative: &str,
	errors: &mut Vec<String>,
) {
	if has_scheme_or_netloc(href) {
		return;
	}
	let (without_fragment, fragment) = match href.split_once('#') {
		Some((path, fragment)) => (path, Some(fragment)),
		None => (href, None),
	};
	// urlsplit separates the query, so query strings never affect the local path.
	let path_part = without_fragment.split('?').next().unwrap_or("");
	let local = percent_decode(path_part);
	let base = page_path.parent().unwrap_or(site);
	let target = if local.starts_with('/') {
		normalize(&site.join(local.trim_start_matches('/')))
	} else if local.is_empty() {
		page_path.to_path_buf()
	} else {
		normalize(&base.join(&local))
	};
	let target = if target.is_dir() { target.join("index.html") } else { target };
	let key = target.canonicalize().unwrap_or_else(|_| target.clone());
	if !key.exists() {
		errors.push(format!("{relative}: {href}"));
		return;
	}
	if let (Some(fragment), Some(index)) = (fragment, by_canonical.get(&key)) {
		let fragment = percent_decode(fragment);
		if !entries[*index].page.ids.contains(&fragment) {
			errors.push(format!("{relative}: missing fragment {href}"));
		}
	}
}

fn check_editor(site: &Path, errors: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
	let editor_path = site.join("config-editor/index.html");
	let editor_html = std::fs::read_to_string(&editor_path)?;
	for marker in ["googletagmanager", "google-analytics", "gtag("] {
		if editor_html.contains(marker) {
			return Err("Editor must not include third-party analytics".into());
		}
	}
	for marker in ["md-header", "md-main", "iframe", "app.mjs"] {
		if editor_html.contains(marker) {
			return Err("Editor must be a standalone Svelte/WASM application".into());
		}
	}

	let page = parse_html(&editor_html);
	let assets: Vec<&String> = page
		.assets
		.iter()
		.filter(|asset| [".js", ".wasm", ".css"].iter().any(|ext| asset.ends_with(ext)))
		.collect();
	if !assets.iter().any(|asset| asset.ends_with(".js")) {
		return Err("Application entry missing".into());
	}

	let assets_dir = site.join("config-editor/assets");
	let mut wasm_assets = Vec::new();
	let mut bundles = Vec::new();
	for item in std::fs::read_dir(&assets_dir)? {
		let path = item?.path();
		match path.extension().and_then(|value| value.to_str()) {
			Some("wasm") => wasm_assets.push(path),
			Some("js") => bundles.push(path),
			_ => {}
		}
	}
	if wasm_assets.is_empty() {
		return Err("Rust WASM engine missing".into());
	}
	let referenced = wasm_assets.iter().any(|wasm| {
		let name = wasm.file_name().and_then(|value| value.to_str()).unwrap_or_default();
		bundles.iter().any(|bundle| {
			std::fs::read_to_string(bundle).map(|text| text.contains(name)).unwrap_or(false)
		})
	});
	if !referenced {
		return Err("WASM engine is not referenced by the application".into());
	}

	let editor_dir = editor_path.parent().unwrap_or(site);
	for asset in assets {
		if has_scheme_or_netloc(asset) {
			return Err("Editor must use local assets".into());
		}
		let target = if asset.starts_with('/') {
			site.join(asset.trim_start_matches('/'))
		} else {
			editor_dir.join(asset)
		};
		if !target.is_file() {
			errors.push(format!("Missing editor asset: {asset}"));
		}
	}
	Ok(())
}

fn collect(root: &Path, dir: &Path, entries: &mut Vec<Entry>) -> Result<(), Box<dyn Error>> {
	for item in std::fs::read_dir(dir)? {
		let path = item?.path();
		if path.is_dir() {
			collect(root, &path, entries)?;
		} else if path.extension().and_then(|value| value.to_str()) == Some("html") {
			let text = std::fs::read_to_string(&path)?;
			let canonical = path.canonicalize()?;
			let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().into_owned();
			entries.push(Entry { canonical, relative, page: parse_html(&text) });
		}
	}
	Ok(())
}

fn parse_html(text: &str) -> Page {
	let bytes = text.as_bytes();
	let mut page = Page::default();
	let mut i = 0;
	while i < bytes.len() {
		if bytes[i] != b'<' {
			i += 1;
			continue;
		}
		if text[i..].starts_with("<!--") {
			i = text[i + 4..].find("-->").map(|end| i + 4 + end + 3).unwrap_or(bytes.len());
			continue;
		}
		if i + 1 < bytes.len() && (bytes[i + 1] == b'!' || bytes[i + 1] == b'?') {
			i = text[i..].find('>').map(|end| i + end + 1).unwrap_or(bytes.len());
			continue;
		}
		if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
			i = text[i..].find('>').map(|end| i + end + 1).unwrap_or(bytes.len());
			continue;
		}

		let mut j = i + 1;
		while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-' || bytes[j] == b':') {
			j += 1;
		}
		let tag = text[i + 1..j].to_ascii_lowercase();
		let (attributes, next) = parse_attributes(text, j);
		i = next;

		for (name, value) in &attributes {
			if name == "id" {
				page.ids.insert(decode_entities(value));
			}
			if (tag == "a" && name == "href") || name == "src" || name == "href" {
				page.assets.push(decode_entities(value));
			}
		}
		if let Some((_, value)) = attributes.iter().find(|(name, _)| tag == "a" && name == "href") {
			page.links.push(decode_entities(value));
		}
		if tag == "script" || tag == "style" {
			if let Some(offset) = find_ignore_ascii_case(&text[i..], &format!("</{tag}")) {
				i += offset;
			}
		}
	}
	page
}

fn parse_attributes(text: &str, mut k: usize) -> (Vec<(String, String)>, usize) {
	let bytes = text.as_bytes();
	let mut attributes = Vec::new();
	loop {
		while k < bytes.len() && (bytes[k] as char).is_whitespace() {
			k += 1;
		}
		if k >= bytes.len() {
			break;
		}
		if bytes[k] == b'>' {
			k += 1;
			break;
		}
		if bytes[k] == b'/' {
			k += 1;
			continue;
		}
		let name_start = k;
		while k < bytes.len()
			&& !(bytes[k] as char).is_whitespace()
			&& bytes[k] != b'='
			&& bytes[k] != b'>'
			&& bytes[k] != b'/'
		{
			k += 1;
		}
		let name = text[name_start..k].to_ascii_lowercase();
		while k < bytes.len() && (bytes[k] as char).is_whitespace() {
			k += 1;
		}
		let mut value = String::new();
		if k < bytes.len() && bytes[k] == b'=' {
			k += 1;
			while k < bytes.len() && (bytes[k] as char).is_whitespace() {
				k += 1;
			}
			if k < bytes.len() && (bytes[k] == b'"' || bytes[k] == b'\'') {
				let quote = bytes[k];
				k += 1;
				let value_start = k;
				while k < bytes.len() && bytes[k] != quote {
					k += 1;
				}
				value = text[value_start..k].to_string();
				if k < bytes.len() {
					k += 1;
				}
			} else {
				let value_start = k;
				while k < bytes.len() && !(bytes[k] as char).is_whitespace() && bytes[k] != b'>' {
					k += 1;
				}
				value = text[value_start..k].to_string();
			}
		}
		attributes.push((name, value));
	}
	(attributes, k)
}

fn repo_root() -> PathBuf {
	if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
		let root = Path::new(&dir).join("..").join("..");
		if let Ok(root) = root.canonicalize() {
			return root;
		}
	}
	std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn normalize(path: &Path) -> PathBuf {
	let mut result = PathBuf::new();
	for component in path.components() {
		match component {
			Component::ParentDir => {
				result.pop();
			}
			Component::CurDir => {}
			other => result.push(other),
		}
	}
	result
}

fn has_scheme_or_netloc(href: &str) -> bool {
	if href.starts_with("//") {
		return true;
	}
	match href.find(':') {
		Some(colon) => href[..colon].find('/').is_none(),
		None => false,
	}
}

fn find_ignore_ascii_case(haystack: &str, needle: &str) -> Option<usize> {
	haystack
		.as_bytes()
		.windows(needle.len())
		.position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn decode_entities(value: &str) -> String {
	let mut output = String::with_capacity(value.len());
	let mut rest = value;
	while let Some(index) = rest.find('&') {
		output.push_str(&rest[..index]);
		rest = &rest[index..];
		if let Some(end) = rest.find(';') {
			let entity = &rest[1..end];
			if let Some(decoded) = decode_entity(entity) {
				output.push(decoded);
				rest = &rest[end + 1..];
				continue;
			}
		}
		output.push('&');
		rest = &rest[1..];
	}
	output.push_str(rest);
	output
}

fn decode_entity(entity: &str) -> Option<char> {
	if let Some(digits) = entity.strip_prefix('#') {
		let (radix, digits) = match digits.strip_prefix('x').or_else(|| digits.strip_prefix('X')) {
			Some(hex) => (16, hex),
			None => (10, digits),
		};
		return u32::from_str_radix(digits, radix).ok().and_then(char::from_u32);
	}
	match entity {
		"amp" => Some('&'),
		"lt" => Some('<'),
		"gt" => Some('>'),
		"quot" => Some('"'),
		"apos" => Some('\''),
		_ => None,
	}
}

fn percent_decode(value: &str) -> String {
	let bytes = value.as_bytes();
	let mut output = Vec::with_capacity(bytes.len());
	let mut i = 0;
	while i < bytes.len() {
		if bytes[i] == b'%' && i + 2 < bytes.len() {
			if let Ok(byte) = u8::from_str_radix(&value[i + 1..i + 3], 16) {
				output.push(byte);
				i += 3;
				continue;
			}
		}
		output.push(bytes[i]);
		i += 1;
	}
	String::from_utf8_lossy(&output).into_owned()
}
