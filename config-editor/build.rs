fn main() {
	println!("cargo:rerun-if-env-changed=CONFIG_SCHEMA");
	let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default());
	let configured = std::env::var_os("CONFIG_SCHEMA").map(std::path::PathBuf::from);
	let entries = if let Some(source) = configured {
		let source = if source.is_absolute() { source } else { root.join(source) };
		let name = source
			.file_stem()
			.and_then(|value| value.to_str())
			.unwrap_or("default")
			.to_owned();
		vec![(name.clone(), vec![(name.clone(), name)], source)]
	} else {
		let manifest = root.join("schema/schemas.txt");
		println!("cargo:rerun-if-changed={}", manifest.display());
		std::fs::read_to_string(&manifest)
			.unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest.display()))
			.lines()
			.filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
			.map(|line| {
				let parts = line.split('|').map(str::trim).collect::<Vec<_>>();
				if parts.len() < 3 {
					panic!("invalid schema manifest entry");
				}
				let name = parts[0];
				let source = parts[parts.len() - 1];
				let source_path = std::path::Path::new(source);
				if name.is_empty()
					|| !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
					|| source.is_empty()
					|| source_path.components().count() != 1
					|| source_path.extension().and_then(|value| value.to_str()) != Some("xml")
				{
					panic!("invalid schema manifest entry");
				}
				let labels = parts[1..parts.len() - 1]
					.iter()
					.map(|cell| {
						let (locale, label) = cell.split_once(':').unwrap_or_else(|| panic!("invalid label cell"));
						if locale.is_empty()
							|| label.is_empty()
							|| !locale.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
						{
							panic!("invalid label cell");
						}
						(locale.to_owned(), label.to_owned())
					})
					.collect::<Vec<_>>();
				if labels.is_empty() {
					panic!("schema manifest entry requires a label");
				}
				(name.to_owned(), labels, root.join("schema").join(source_path))
			})
			.collect::<Vec<_>>()
	};
	if entries.is_empty() {
		panic!("at least one configuration schema is required");
	}
	let mut names = std::collections::BTreeSet::new();
	let mut generated =
		String::from("#[allow(clippy::type_complexity)]\npub const SOURCES: &[(&str, &[(&str, &str)], &str)] = &[\n");
	for (name, labels, source) in entries {
		if !names.insert(name.clone()) {
			panic!("duplicate configuration schema name");
		}
		println!("cargo:rerun-if-changed={}", source.display());
		let labels = labels
			.iter()
			.map(|(locale, label)| format!("({locale:?}, {label:?})"))
			.collect::<Vec<_>>()
			.join(", ");
		generated.push_str(&format!(
			"\t({name:?}, &[{labels}], include_str!({source:?})),\n",
			source = source.to_string_lossy()
		));
	}
	generated.push_str("];\n");
	let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default());
	std::fs::write(out.join("embedded_schemas.rs"), generated).expect("cannot write embedded schema registry");
}
