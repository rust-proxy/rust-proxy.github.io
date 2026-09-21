use config_generator::{
	dsl::Document,
	model::build_configs,
	schema::{State, document_for, schemas},
};

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn rendered_toml(config: &config_generator::dsl::DescriptionConfig, selected: &[(&str, &str)]) -> String {
	config
		.formats
		.iter()
		.find(|format| format.name == "toml")
		.into_iter()
		.flat_map(|format| &format.lines)
		.filter(|line| {
			line.conditions.iter().all(|(name, value)| {
				selected
					.iter()
					.find(|(candidate, _)| *candidate == name)
					.is_some_and(|(_, chosen)| *chosen == value)
			})
		})
		.map(|line| line.text.as_str())
		.collect::<Vec<_>>()
		.join("\n")
}

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
	assert_eq!(
		server.config_description.as_ref().map(|desc| desc.configs[0].name.as_str()),
		Some("server")
	);

	let client = document_for("tuic-client")?;
	assert!(client.ui.mode_field.is_none());
	assert_eq!(
		client.exports.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(),
		["client"]
	);
	assert!(client.fields.iter().any(|field| field.key == "host"));
	assert!(!client.fields.iter().any(|field| field.key == "listen"));
	assert_eq!(
		client.config_description.as_ref().map(|desc| desc.configs[0].name.as_str()),
		Some("client")
	);
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
		let description = document.config_description.as_ref().ok_or("missing config description")?;
		for entry in &description.configs {
			let Some(config) = configs.get(entry.name.as_str()) else {
				continue;
			};
			let yaml = entry.render_annotated(config, "yaml")?;
			assert!(
				yaml.iter().filter(|line| !line.description.is_empty()).count() > 5,
				"{schema} {}: generated config is not annotated",
				entry.name
			);
			assert!(
				yaml.iter().any(|line| line.description.contains(marker)),
				"{schema} {}: documented descriptions are missing",
				entry.name
			);
			let toml_text = entry
				.render_annotated(config, "toml")?
				.iter()
				.map(|line| line.text.clone())
				.collect::<Vec<_>>()
				.join("\n");
			assert_eq!(serde_json::to_value(toml::from_str::<toml::Value>(&toml_text)?)?, *config);
			let json_text = entry
				.render_annotated(config, "json")?
				.iter()
				.map(|line| line.text.clone())
				.collect::<Vec<_>>()
				.join("\n");
			assert_eq!(serde_json::from_str::<serde_json::Value>(&json_text)?, *config);
		}
	}
	Ok(())
}

#[test]
fn structured_descriptions_render_valid_toml_across_selector_choices() -> Result {
	for schema in ["tuic-server", "tuic-client"] {
		let document = document_for(schema)?;
		let description = document.config_description.as_ref().ok_or("missing config description")?;
		for config in &description.configs {
			let defaults = config
				.selectors
				.iter()
				.map(|selector| (selector.name.as_str(), selector.default.as_str()))
				.collect::<Vec<_>>();
			toml::from_str::<toml::Value>(&rendered_toml(config, &defaults))?;
			for selector in &config.selectors {
				for (choice, _) in &selector.choices {
					let mut selected = defaults.clone();
					let Some((_, value)) = selected.iter_mut().find(|(name, _)| *name == selector.name) else {
						return Err("selector default is missing".into());
					};
					*value = choice;
					toml::from_str::<toml::Value>(&rendered_toml(config, &selected))?;
				}
			}
		}
	}
	Ok(())
}
