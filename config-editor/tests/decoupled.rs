use config_editor::{dsl::Document, model::build_configs, schema::State};
use serde_json::json;

const SOURCE: &str = include_str!("../schema/example.xml");
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn unrelated_description_drives_the_complete_engine() -> Result {
	let doc = Document::parse(SOURCE)?;
	let mut state = State::new(&doc);
	assert!(doc.validate(&state.data).is_empty());
	assert_eq!(doc.ui.brand, "Notebook");
	assert_eq!(doc.ui.sections.len(), 2);
	assert_eq!(doc.exports.len(), 3);
	let config = build_configs(&doc, &state.data)?;
	assert_eq!(config["snapshot"]["items"][0]["name"], "item");
	assert_eq!(config["selected"]["item"], "item");
	assert!(config.get("audit").is_none());
	let preview = doc.preview_lines("snapshot", &config["snapshot"], "yaml")?;
	assert_eq!(preview[0].text, "title: \"example\"");
	assert_eq!(preview[0].description, "清单所属项目的名称。");
	assert_eq!(preview[1].text, "items:");
	let toml = doc
		.preview_lines("snapshot", &config["snapshot"], "toml")?
		.iter()
		.map(|line| line.text.as_str())
		.collect::<Vec<_>>()
		.join("\n");
	let parsed: toml::Value = toml::from_str(&toml)?;
	assert_eq!(parsed["title"].as_str(), Some("example"));
	assert_eq!(parsed["items"][0]["name"].as_str(), Some("item"));
	assert_eq!(doc.exports[0].command("toml"), "notebook --input snapshot.toml");
	state.data["entries"][0]["token"] = "ephemeral".into();
	let config = build_configs(&doc, &state.data)?;
	assert_eq!(doc.redact(&config)?["snapshot"]["items"][0]["secret"], "••••••••");
	assert_eq!(config["snapshot"]["items"][0]["secret"], "ephemeral");
	state.data["confirm"] = true.into();
	assert!(doc.write_field(&mut state.data, "project", "example"));
	assert_eq!(state.data["confirm"], true, "unchanged inputs do not run resets");
	assert!(doc.write_field(&mut state.data, "project", "renamed"));
	assert_eq!(state.data["confirm"], false);
	state.data["first"] = "20".into();
	assert!(doc.validate(&state.data).contains_key("last"));
	assert!(build_configs(&doc, &state.data).is_err());
	state.data["project"] = String::new().into();
	assert!(doc.validate(&state.data).contains_key("project"));
	assert!(build_configs(&doc, &state.data).is_err());
	let preview = doc.project_preview(&state.data).ok_or("preview")?;
	assert_eq!(preview["snapshot"]["title"], "<placeholder>");
	assert_eq!(preview["snapshot"]["items"][0]["name"], "item");
	Ok(())
}

#[test]
fn generated_config_is_annotated_from_the_documented_tree() -> Result {
	let doc = Document::parse(SOURCE)?;
	let state = State::new(&doc);
	let config = build_configs(&doc, &state.data)?;
	let snapshot = &config["snapshot"];
	let yaml = doc.preview_lines("snapshot", snapshot, "yaml")?;
	let find = |prefix: &str| {
		yaml.iter()
			.find(|line| line.text.trim_start().starts_with(prefix))
			.map(|line| line.description.as_str())
	};
	assert!(find("title:").is_some_and(|description| description.contains("项目")));
	assert!(find("name:").is_some_and(|description| description.contains("唯一")));
	assert!(find("secret:").is_some_and(|description| description.contains("敏感")));
	for format in ["json", "toml"] {
		let text = doc
			.preview_lines("snapshot", snapshot, format)?
			.iter()
			.map(|line| line.text.as_str())
			.collect::<Vec<_>>()
			.join("\n");
		let parsed: serde_json::Value = match format {
			"json" => serde_json::from_str(&text)?,
			_ => serde_json::to_value(toml::from_str::<toml::Value>(&text)?)?,
		};
		assert_eq!(&parsed, snapshot, "annotated {format} must match the export");
	}
	Ok(())
}

#[test]
fn arbitrary_collection_ids_selection_visibility_and_minimum() -> Result {
	let doc = Document::parse(SOURCE)?;
	let mut state = State::new(&doc);
	let first = state.rows("entries")[0];
	let second = state.add(&doc, "entries")?;
	let third = state.add(&doc, "entries")?;
	state.data["entries"][1]["id"] = "second".into();
	state.data["entries"][2]["id"] = "third".into();
	state.data["chosen"] = 2.into();
	assert!(state.remove(&doc, "entries", first));
	assert_eq!(state.data["chosen"], 1);
	assert_eq!(state.row("entries", third).and_then(|v| v["id"].as_str()), Some("third"));
	assert_eq!(state.rows("entries"), vec![second, third]);
	assert!(state.remove(&doc, "entries", third));
	assert_eq!(state.data["chosen"], 0);
	assert!(!state.remove(&doc, "entries", second));
	assert_eq!(
		state.data["entries"][0]["id"], "second",
		"the input id is not reserved for UI identity"
	);
	let last = state.add(&doc, "entries")?;
	state.data["entries"][1]["id"] = " SECOND ".into();
	assert!(doc.validate(&state.data).contains_key("entries.1.id"));
	state.data["deployment"] = "one".into();
	assert!(doc.validate(&state.data).is_empty());
	assert!(!doc.collections[0].row_visible(&doc, &state.data, 1)?);
	assert!(!state.remove(&doc, "entries", last));
	assert!(state.add(&doc, "entries").is_err());
	assert!(build_configs(&doc, &state.data)?.get("snapshot").is_none());
	state.data["chosen"] = 100.into();
	assert!(doc.validate(&state.data).contains_key("chosen"));
	Ok(())
}

#[test]
fn generation_is_declared_per_field_and_never_overwrites_other_values() -> Result {
	let doc = Document::parse(SOURCE)?;
	let fields = &doc.collections[0].fields;
	assert!(fields[0].generator.is_none());
	let uuid = fields[2].generator.as_ref().ok_or("missing generator")?;
	let value = uuid.encode(&[0; 16])?;
	let parsed = uuid::Uuid::parse_str(&value)?;
	assert_eq!(parsed.get_version_num(), 4);
	assert_eq!(parsed.get_variant(), uuid::Variant::RFC4122);
	let hex = fields[3].generator.as_ref().ok_or("missing generator")?;
	assert_eq!(hex.byte_count(), 12);
	assert_eq!(hex.encode(&[0xab; 12])?, "ab".repeat(12));
	assert!(hex.encode(&[0; 8]).is_err());
	Ok(())
}

#[test]
fn schema_extensions_fail_closed() {
	for (from, to) in [
		("rule=\"text\"", "rule=\"missing\""),
		("section=\"general\"", "section=\"missing\""),
		("selected-by=\"chosen\"", "selected-by=\"project\""),
		("all-when=\"all\"", "all-when=\"missing\""),
		("collection=\"entries\"", "collection=\"missing\""),
		("target=\"confirm\"", "target=\"missing\""),
		("mode-field=\"deployment\"", "mode-field=\"project\""),
		("generator=\"hex\" bytes=\"12\"", "generator=\"hex\" bytes=\"0\""),
		("generator=\"uuid-v4\"", "generator=\"execute\""),
		("min=\"1\" max=\"100\"", "min=\"100\" max=\"1\""),
		("op=\"gte\"", "op=\"execute\""),
		("filename=\"snapshot\"", "filename=\"../snapshot\""),
		("from=\"/project\"", "from=\"/project\" unknown=\"true\""),
	] {
		assert!(
			Document::parse(&SOURCE.replace(from, to)).is_err(),
			"accepted invalid binding: {from}"
		);
	}
}

#[test]
fn config_desc_and_legacy_lines_are_rejected() {
	let described = SOURCE.replace(
		"<outputs>",
		"<config-desc title=\"d\"><config name=\"out\" label=\"o\" filename=\"out\"><string name=\"x\" value=\"a\" description=\"x\"/></config></config-desc><outputs>",
	);
	assert!(Document::parse(&described).is_err(), "config-desc must be rejected");
	let legacy = SOURCE.replace(
		"<string name=\"title\" from=\"/project\" description=\"清单所属项目的名称。\"/>",
		"<line yaml=\"title: example\" description=\"清单所属项目的名称。\"/>",
	);
	assert!(Document::parse(&legacy).is_err(), "legacy line descriptions must be rejected");
}

#[test]
fn collection_level_constraints_use_declared_error_keys() -> Result {
	let xml = SOURCE
		.replace(
			"</validators>",
			"<validator name='limit' kind='length' max='1' message='Too many items'/></validators>",
		)
		.replace(
			"</rules>",
			"<assert key='entries' message='Too many items'><valid from='/entries' rule='limit'/></assert></rules>",
		);
	let doc = Document::parse(&xml)?;
	let mut state = State::new(&doc);
	assert!(doc.validate(&state.data).is_empty());
	state.add(&doc, "entries")?;
	state.data["entries"][1]["id"] = "second".into();
	assert_eq!(
		doc.validate(&state.data).get("entries").map(String::as_str),
		Some("Too many items")
	);
	assert!(build_configs(&doc, &state.data).is_err());
	Ok(())
}

#[test]
fn malformed_state_is_not_coerced_or_exported() -> Result {
	let doc = Document::parse(SOURCE)?;
	for (field, value) in [
		("confirm", json!("false")),
		("chosen", json!(-1)),
		("entries", json!({})),
		("project", json!(null)),
		("deployment", json!("unknown")),
	] {
		let mut root = doc.defaults();
		root[field] = value;
		assert!(!doc.validate(&root).is_empty());
		assert!(build_configs(&doc, &root).is_err());
	}
	Ok(())
}

#[test]
fn production_code_contains_no_product_identifiers() {
	for source in [
		include_str!("../src/session.rs"),
		include_str!("../src/session/view.rs"),
		include_str!("../src/wasm.rs"),
		include_str!("../ui/App.svelte"),
		include_str!("../ui/main.ts"),
		include_str!("../ui/types.ts"),
		include_str!("../ui/bridge/engine.ts"),
		include_str!("../ui/state/context.ts"),
		include_str!("../ui/state/session.svelte.ts"),
		include_str!("../ui/state/workbench.svelte.ts"),
		include_str!("../ui/state/theme.svelte.ts"),
		include_str!("../ui/state/viewport.svelte.ts"),
		include_str!("../ui/state/url.ts"),
		include_str!("../ui/lib/dom.ts"),
		include_str!("../ui/lib/highlight.ts"),
		include_str!("../ui/components/AppHeader.svelte"),
		include_str!("../ui/components/SectionNav.svelte"),
		include_str!("../ui/components/FieldSearch.svelte"),
		include_str!("../ui/components/FormSection.svelte"),
		include_str!("../ui/components/FieldControl.svelte"),
		include_str!("../ui/components/CollectionEditor.svelte"),
		include_str!("../ui/components/CollectionRow.svelte"),
		include_str!("../ui/components/PreviewPanel.svelte"),
		include_str!("../ui/components/PreviewDocument.svelte"),
		include_str!("../ui/components/ExportActions.svelte"),
		include_str!("../ui/components/Notices.svelte"),
		include_str!("../src/schema.rs"),
		include_str!("../src/validation.rs"),
		include_str!("../src/model.rs"),
		include_str!("../src/dsl.rs"),
		include_str!("../src/dsl/preview.rs"),
		include_str!("../src/dsl/parser.rs"),
		include_str!("../src/dsl/metadata.rs"),
		include_str!("../src/dsl/rules.rs"),
		include_str!("../src/dsl/wire.rs"),
		include_str!("../index.html"),
		include_str!("../build.rs"),
		include_str!("../Cargo.toml"),
	] {
		for name in [
			"tuic",
			"quinn",
			"socks",
			"acme",
			"tlsMode",
			"activeUser",
			"forwards",
			"maxBackoff",
		] {
			assert!(
				!source.to_lowercase().contains(&name.to_lowercase()),
				"product identifier {name} leaked into engine"
			);
		}
	}
}
