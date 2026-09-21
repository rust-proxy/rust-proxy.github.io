//! Annotated preview: walks the declared output tree with a projected value and renders
//! YAML, TOML, or JSON with one description per line. Descriptions come from the output
//! nodes and, for enums, from the selected input option.
use serde::Serialize;
use serde_json::Value;

use super::{DslError, Element, InputField};

#[derive(Debug, Clone)]
enum Node {
	Scalar {
		name: Option<String>,
		value: Value,
		description: String,
	},
	Object {
		name: Option<String>,
		description: String,
		children: Vec<Node>,
	},
	Array {
		name: Option<String>,
		description: String,
		items: Vec<Node>,
	},
}

impl Node {
	fn name(&self) -> Option<&str> {
		match self {
			Self::Scalar { name, .. } | Self::Object { name, .. } | Self::Array { name, .. } => name.as_deref(),
		}
	}
	fn set_name(&mut self, value: String) {
		match self {
			Self::Scalar { name, .. } | Self::Object { name, .. } | Self::Array { name, .. } => *name = Some(value),
		}
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct PreviewLine {
	pub text: String,
	pub description: String,
}

/// Renders one top-level output as annotated lines for the requested format.
pub(super) fn render(
	export: &Element,
	value: &Value,
	fields: &[InputField],
	format: &str,
) -> Result<Vec<PreviewLine>, DslError> {
	let node = annotate(export, value, fields)?;
	let Node::Object { children, .. } = node else {
		return Err(export.error("输出必须是对象"));
	};
	match format {
		"yaml" => Ok(render_yaml(&children)),
		"toml" => Ok(render_toml(&children)),
		"json" => Ok(render_json(&children)),
		_ => Err(DslError("不支持的预览格式".into())),
	}
}

fn annotate(node: &Element, value: &Value, fields: &[InputField]) -> Result<Node, DslError> {
	match (node.tag.as_str(), value) {
		("object", Value::Object(entries)) => {
			let children = entries
				.iter()
				.map(|(key, child)| {
					let template = node
						.children
						.iter()
						.find(|candidate| candidate.attr("name") == Some(key.as_str()))
						.ok_or_else(|| node.error("输出含有未声明的字段"))?;
					annotate(template, child, fields)
				})
				.collect::<Result<Vec<_>, _>>()?;
			Ok(Node::Object {
				name: node.attr("name").map(str::to_owned),
				description: node.attr("description").unwrap_or_default().into(),
				children,
			})
		}
		("list", Value::Array(items)) => {
			let template = node.single()?;
			let children = items
				.iter()
				.map(|item| annotate(template, item, fields))
				.collect::<Result<Vec<_>, _>>()?;
			Ok(Node::Array {
				name: node.attr("name").map(str::to_owned),
				description: node.attr("description").unwrap_or_default().into(),
				items: children,
			})
		}
		("record", Value::Object(entries)) => {
			let template = node.single()?;
			let children = entries
				.iter()
				.map(|(key, child)| {
					let mut item = annotate(template, child, fields)?;
					item.set_name(key.clone());
					Ok(item)
				})
				.collect::<Result<Vec<_>, DslError>>()?;
			Ok(Node::Object {
				name: node.attr("name").map(str::to_owned),
				description: node.attr("description").unwrap_or_default().into(),
				children,
			})
		}
		("string" | "boolean" | "integer" | "enum", _) => Ok(Node::Scalar {
			name: node.attr("name").map(str::to_owned),
			value: value.clone(),
			description: scalar_description(node, value, fields),
		}),
		_ => Err(node.error("输出结构与生成值不匹配")),
	}
}

fn scalar_description(node: &Element, value: &Value, fields: &[InputField]) -> String {
	if node.tag == "enum"
		&& let Some(key) = node.attr("options")
		&& let Some(field) = fields.iter().find(|field| field.key == key)
		&& let Some(value) = value.as_str()
		&& let Some(description) = field.option_descriptions.get(value)
	{
		return description.clone();
	}
	node.attr("description").unwrap_or_default().into()
}

fn preview_line(text: String, description: &str) -> PreviewLine {
	PreviewLine {
		text,
		description: description.into(),
	}
}

fn scalar(value: &Value) -> String {
	match value {
		Value::String(value) => serde_json::to_string(value).unwrap_or_else(|_| "\"\"".into()),
		_ => value.to_string(),
	}
}

fn yaml_key(name: &str) -> String {
	if !name.is_empty()
		&& name
			.bytes()
			.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
	{
		name.into()
	} else {
		serde_json::to_string(name).unwrap_or_else(|_| "\"\"".into())
	}
}

fn render_yaml(nodes: &[Node]) -> Vec<PreviewLine> {
	let mut lines = Vec::new();
	for node in nodes {
		yaml_node(node, 0, false, &mut lines);
	}
	lines
}

fn yaml_node(node: &Node, depth: usize, array_item: bool, lines: &mut Vec<PreviewLine>) {
	let pad = "  ".repeat(depth);
	let prefix = if array_item { "- " } else { "" };
	match node {
		Node::Scalar {
			name,
			value,
			description,
		} => {
			let text = match name {
				Some(name) => format!("{pad}{prefix}{}: {}", yaml_key(name), scalar(value)),
				None => format!("{pad}{prefix}{}", scalar(value)),
			};
			lines.push(preview_line(text, description));
		}
		Node::Object {
			name,
			description,
			children,
		} => {
			let text = match name {
				Some(name) => format!("{pad}{prefix}{}:", yaml_key(name)),
				None => format!("{pad}-"),
			};
			lines.push(preview_line(text, description));
			for child in children {
				yaml_node(child, depth + 1, false, lines);
			}
		}
		Node::Array {
			name,
			description,
			items,
		} => {
			let Some(name) = name else { return };
			let text = if items.is_empty() {
				format!("{pad}{prefix}{}: []", yaml_key(name))
			} else {
				format!("{pad}{prefix}{}:", yaml_key(name))
			};
			lines.push(preview_line(text, description));
			for item in items {
				yaml_node(item, depth + 1, true, lines);
			}
		}
	}
}

fn toml_key(name: &str) -> String {
	if !name.is_empty()
		&& name
			.bytes()
			.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
	{
		name.into()
	} else {
		serde_json::to_string(name).unwrap_or_else(|_| "\"\"".into())
	}
}

fn toml_path(path: &[String]) -> String {
	path.iter().map(|part| toml_key(part)).collect::<Vec<_>>().join(".")
}

fn render_toml(nodes: &[Node]) -> Vec<PreviewLine> {
	let mut lines = Vec::new();
	toml_members(nodes, &[], &mut lines);
	lines
}

fn toml_members(nodes: &[Node], path: &[String], lines: &mut Vec<PreviewLine>) {
	for node in nodes {
		match node {
			Node::Scalar {
				name: Some(name),
				value,
				description,
			} => lines.push(preview_line(format!("{} = {}", toml_key(name), scalar(value)), description)),
			Node::Array {
				name: Some(name),
				description,
				items,
			} if items.first().is_none_or(|item| !matches!(item, Node::Object { .. })) => {
				lines.push(preview_line(format!("{} = [", toml_key(name)), description));
				for item in items {
					if let Node::Scalar { value, description, .. } = item {
						lines.push(preview_line(format!("  {},", scalar(value)), description));
					}
				}
				lines.push(preview_line("]".into(), description));
			}
			_ => {}
		}
	}
	for node in nodes {
		match node {
			Node::Object {
				name: Some(name),
				description,
				children,
			} => {
				let mut child_path = path.to_vec();
				child_path.push(name.clone());
				lines.push(preview_line(format!("[{}]", toml_path(&child_path)), description));
				toml_members(children, &child_path, lines);
			}
			Node::Array {
				name: Some(name),
				description,
				items,
			} if items.first().is_some_and(|item| matches!(item, Node::Object { .. })) => {
				let mut item_path = path.to_vec();
				item_path.push(name.clone());
				for item in items {
					let Node::Object {
						description: item_description,
						children,
						..
					} = item
					else {
						continue;
					};
					let text_description = if item_description.is_empty() {
						description
					} else {
						item_description
					};
					lines.push(preview_line(format!("[[{}]]", toml_path(&item_path)), text_description));
					toml_members(children, &item_path, lines);
				}
			}
			_ => {}
		}
	}
}

fn json_scalar(value: &Value) -> String {
	match value {
		Value::String(value) => serde_json::to_string(value)
			.unwrap_or_else(|_| "\"\"".into())
			.replace('\u{7f}', "\\u007f")
			.replace('\u{85}', "\\u0085")
			.replace('\u{2028}', "\\u2028")
			.replace('\u{2029}', "\\u2029"),
		_ => value.to_string(),
	}
}

fn render_json(nodes: &[Node]) -> Vec<PreviewLine> {
	let mut lines = vec![preview_line("{".into(), "")];
	for (index, node) in nodes.iter().enumerate() {
		let suffix = if index + 1 < nodes.len() { "," } else { "" };
		json_node(node, 1, suffix, &mut lines);
	}
	lines.push(preview_line("}".into(), ""));
	lines
}

fn json_node(node: &Node, depth: usize, suffix: &str, lines: &mut Vec<PreviewLine>) {
	let pad = "  ".repeat(depth);
	let key = match node.name() {
		Some(name) => format!("{}: ", serde_json::to_string(name).unwrap_or_else(|_| "\"\"".into())),
		None => String::new(),
	};
	match node {
		Node::Scalar { value, description, .. } => {
			lines.push(preview_line(format!("{pad}{key}{}{suffix}", json_scalar(value)), description))
		}
		Node::Object {
			description, children, ..
		} => {
			lines.push(preview_line(format!("{pad}{key}{{"), description));
			for (index, child) in children.iter().enumerate() {
				let inner = if index + 1 < children.len() { "," } else { "" };
				json_node(child, depth + 1, inner, lines);
			}
			lines.push(preview_line(format!("{pad}}}{suffix}"), ""));
		}
		Node::Array { description, items, .. } => {
			lines.push(preview_line(format!("{pad}{key}["), description));
			for (index, item) in items.iter().enumerate() {
				let inner = if index + 1 < items.len() { "," } else { "" };
				json_node(item, depth + 1, inner, lines);
			}
			lines.push(preview_line(format!("{pad}]{suffix}"), ""));
		}
	}
}
