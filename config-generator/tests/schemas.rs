use config_generator::{
	dsl::Document,
	model::build_configs,
	schema::{State, document_for, schemas},
};

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn embedded_application_schemas_are_independent() -> Result {
	assert_eq!(
		schemas()?,
		vec![("tuic-server", "TUIC 服务端"), ("tuic-client", "TUIC 客户端")]
	);

	let server = document_for("tuic-server")?;
	assert!(server.ui.mode_field.is_none());
	assert_eq!(
		server.exports.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(),
		["server"]
	);
	assert!(server.fields.iter().any(|field| field.key == "listen"));
	assert!(!server.fields.iter().any(|field| field.key == "host"));

	let client = document_for("tuic-client")?;
	assert!(client.ui.mode_field.is_none());
	assert_eq!(
		client.exports.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(),
		["client"]
	);
	assert!(client.fields.iter().any(|field| field.key == "host"));
	assert!(!client.fields.iter().any(|field| field.key == "listen"));
	assert!(document_for("missing").is_err());
	Ok(())
}

#[test]
fn split_schemas_preserve_standalone_tuic_outputs() -> Result {
	let legacy = Document::parse(include_str!("fixtures/config.xml"))?;
	for (schema, mode, required, value, output) in [
		("tuic-server", "server", "hostname", "tuic.example.com", "server"),
		("tuic-client", "client", "host", "tuic.example.com", "client"),
	] {
		let current = document_for(schema)?;
		let mut current_state = current.defaults();
		current_state[required] = value.into();
		current_state["users"][0]["uuid"] = "00000000-0000-4000-8000-000000000001".into();
		current_state["users"][0]["password"] = "test-password".into();

		let mut legacy_state = legacy.defaults();
		legacy_state["mode"] = mode.into();
		legacy_state[required] = value.into();
		legacy_state["users"][0]["uuid"] = current_state["users"][0]["uuid"].clone();
		legacy_state["users"][0]["password"] = current_state["users"][0]["password"].clone();

		assert_eq!(
			build_configs(current, &current_state)?[output],
			build_configs(&legacy, &legacy_state)?[output]
		);
	}
	Ok(())
}

#[test]
fn generated_tuic_configs_are_annotated_across_formats() -> Result {
	for (schema, required, value, marker) in [
		("tuic-server", "hostname", "tuic.example.com", "凭据"),
		("tuic-client", "host", "tuic.example.com", "密码"),
	] {
		let document = document_for(schema)?;
		let mut state = State::new(document);
		state.data[required] = value.into();
		state.data["users"][0]["uuid"] = "00000000-0000-4000-8000-000000000001".into();
		state.data["users"][0]["password"] = "test-password".into();
		let configs = build_configs(document, &state.data)?;
		for export in &document.exports {
			let Some(config) = configs.get(export.name.as_str()) else {
				continue;
			};
			let yaml = document.preview_lines(&export.name, config, "yaml")?;
			assert!(
				yaml.iter().filter(|line| !line.description.is_empty()).count() > 5,
				"{schema} {}: generated config is not annotated",
				export.name
			);
			assert!(
				yaml.iter().any(|line| line.description.contains(marker)),
				"{schema} {}: documented descriptions are missing",
				export.name
			);
			let toml_text = document
				.preview_lines(&export.name, config, "toml")?
				.iter()
				.map(|line| line.text.clone())
				.collect::<Vec<_>>()
				.join("\n");
			assert_eq!(serde_json::to_value(toml::from_str::<toml::Value>(&toml_text)?)?, *config);
			let json_text = document
				.preview_lines(&export.name, config, "json")?
				.iter()
				.map(|line| line.text.clone())
				.collect::<Vec<_>>()
				.join("\n");
			assert_eq!(serde_json::from_str::<serde_json::Value>(&json_text)?, *config);
		}
	}
	Ok(())
}
