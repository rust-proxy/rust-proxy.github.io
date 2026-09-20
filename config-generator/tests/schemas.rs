use config_generator::{
	dsl::Document,
	model::build_configs,
	schema::{document_for, schemas},
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
