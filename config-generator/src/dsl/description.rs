use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde_json::Value;

use super::{DslError, Element, parser::attrs};

#[derive(Debug, Clone, Serialize)]
pub struct ConfigDescription {
	pub title: String,
	pub description: String,
	pub configs: Vec<DescriptionConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescriptionConfig {
	pub name: String,
	pub label: String,
	pub filename: String,
	pub selectors: Vec<DescriptionSelector>,
	pub formats: Vec<DescriptionFormat>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescriptionSelector {
	pub name: String,
	pub label: String,
	pub default: String,
	pub choices: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescriptionFormat {
	pub name: &'static str,
	pub label: &'static str,
	pub extension: &'static str,
	pub lines: Vec<DescriptionLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescriptionLine {
	pub text: String,
	pub description: String,
	pub conditions: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
enum DescriptionNode {
	Scalar {
		name: Option<String>,
		value: Value,
		description: String,
		conditions: Vec<(String, String)>,
	},
	Object {
		name: Option<String>,
		description: String,
		conditions: Vec<(String, String)>,
		children: Vec<DescriptionNode>,
	},
	Array {
		name: Option<String>,
		description: String,
		conditions: Vec<(String, String)>,
		items: Vec<DescriptionNode>,
	},
}

impl DescriptionNode {
	fn name(&self) -> Option<&str> {
		match self {
			Self::Scalar { name, .. } | Self::Object { name, .. } | Self::Array { name, .. } => name.as_deref(),
		}
	}
	fn conditions(&self) -> &[(String, String)] {
		match self {
			Self::Scalar { conditions, .. } | Self::Object { conditions, .. } | Self::Array { conditions, .. } => conditions,
		}
	}
}

fn identifier<'a>(node: &'a Element, attr: &str) -> Result<&'a str, DslError> {
	let value = node.required(attr)?;
	if value.is_empty()
		|| !value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
		|| value.as_bytes()[0].is_ascii_digit()
	{
		return Err(node.error("名称必须是静态标识符"));
	}
	Ok(value)
}

pub(super) fn parse(node: Option<&Element>) -> Result<Option<ConfigDescription>, DslError> {
	let Some(node) = node else { return Ok(None) };
	attrs(node, &["title", "description"])?;
	let mut configs = Vec::new();
	let mut config_names = BTreeSet::new();
	for config in &node.children {
		if config.tag != "config" {
			return Err(config.error("config-desc 只接受 config"));
		}
		attrs(config, &["name", "label", "filename"])?;
		let name = identifier(config, "name")?;
		if !config_names.insert(name.to_owned()) {
			return Err(config.error("重复配置名称"));
		}
		let selectors = parse_selectors(config)?;
		let selector_map: BTreeMap<_, _> = selectors.iter().map(|s| (s.name.as_str(), s)).collect();
		let content: Vec<_> = config.children.iter().filter(|child| child.tag != "selector").collect();
		if content.is_empty() {
			return Err(config.error("配置说明至少需要一个节点"));
		}
		let filename = filename(config.required("filename")?, config)?;
		let nodes = content
			.iter()
			.map(|child| parse_node(child, true, &selector_map))
			.collect::<Result<Vec<_>, _>>()?;
		check_duplicate_names(&nodes, &[], config)?;
		let formats = vec![
			DescriptionFormat {
				name: "yaml",
				label: "YAML",
				extension: "yaml",
				lines: render_yaml(&nodes)?,
			},
			DescriptionFormat {
				name: "toml",
				label: "TOML",
				extension: "toml",
				lines: render_toml(&nodes)?,
			},
		];
		configs.push(DescriptionConfig {
			name: name.into(),
			label: config.required("label")?.into(),
			filename,
			selectors,
			formats,
		});
	}
	if configs.is_empty() {
		return Err(node.error("config-desc 至少需要一个 config"));
	}
	Ok(Some(ConfigDescription {
		title: node.required("title")?.into(),
		description: node.attr("description").unwrap_or_default().into(),
		configs,
	}))
}

fn filename(value: &str, node: &Element) -> Result<String, DslError> {
	if value.is_empty() || value.trim() != value || value.contains(['/', '\\', '\r', '\n']) || [".", ".."].contains(&value) {
		return Err(node.error("filename 必须是不含扩展名的安全文件名"));
	}
	Ok(value.into())
}

fn parse_selectors(config: &Element) -> Result<Vec<DescriptionSelector>, DslError> {
	let mut selectors = Vec::new();
	let mut selector_names = BTreeSet::new();
	for child in config.children.iter().filter(|child| child.tag == "selector") {
		attrs(child, &["name", "label", "default"])?;
		let selector_name = identifier(child, "name")?;
		if !selector_names.insert(selector_name.to_owned()) {
			return Err(child.error("重复选择器名称"));
		}
		let mut choices = Vec::new();
		let mut values = BTreeSet::new();
		for choice in &child.children {
			if choice.tag != "choice" || !choice.children.is_empty() {
				return Err(choice.error("selector 只接受空 choice"));
			}
			attrs(choice, &["value", "label"])?;
			let value = identifier(choice, "value")?;
			if !values.insert(value.to_owned()) {
				return Err(choice.error("重复选择值"));
			}
			choices.push((value.into(), choice.required("label")?.into()));
		}
		let default = child.required("default")?;
		if choices.is_empty() || !values.contains(default) {
			return Err(child.error("选择器默认值必须属于非空选项"));
		}
		selectors.push(DescriptionSelector {
			name: selector_name.into(),
			label: child.required("label")?.into(),
			default: default.into(),
			choices,
		});
	}
	Ok(selectors)
}

fn parse_node(
	node: &Element,
	require_name: bool,
	selectors: &BTreeMap<&str, &DescriptionSelector>,
) -> Result<DescriptionNode, DslError> {
	let name = node.attr("name").map(str::to_owned);
	if require_name && name.is_none() {
		return Err(node.error("对象成员必须声明 name"));
	}
	if !require_name && name.is_some() {
		return Err(node.error("数组元素不能声明 name"));
	}
	if let Some(name) = &name
		&& (name.is_empty() || name.contains(['\r', '\n']))
	{
		return Err(node.error("name 不能为空或包含换行"));
	}
	let conditions = parse_conditions(node, selectors)?;
	match node.tag.as_str() {
		"string" | "integer" | "boolean" => {
			attrs(node, &["name", "value", "description", "when"])?;
			if !node.children.is_empty() {
				return Err(node.error("标量节点不能包含子元素"));
			}
			let raw = node.required("value")?;
			let value = match node.tag.as_str() {
				"string" => Value::String(raw.into()),
				"integer" => raw
					.parse::<i64>()
					.map(Value::from)
					.map_err(|_| node.error("integer value 必须是有符号 64 位整数"))?,
				"boolean" => raw
					.parse::<bool>()
					.map(Value::from)
					.map_err(|_| node.error("boolean value 必须为 true 或 false"))?,
				_ => unreachable!(),
			};
			Ok(DescriptionNode::Scalar {
				name,
				value,
				description: node.required("description")?.into(),
				conditions,
			})
		}
		"object" => {
			attrs(node, &["name", "description", "when"])?;
			if node.children.is_empty() {
				return Err(node.error("object 至少需要一个子元素"));
			}
			let children = node
				.children
				.iter()
				.map(|child| parse_node(child, true, selectors))
				.collect::<Result<Vec<_>, _>>()?;
			check_duplicate_names(&children, &conditions, node)?;
			Ok(DescriptionNode::Object {
				name,
				description: description(node, require_name)?,
				conditions,
				children,
			})
		}
		"array" => {
			attrs(node, &["name", "description", "when"])?;
			if node.children.is_empty() {
				return Err(node.error("array 至少需要一个元素"));
			}
			let items = node
				.children
				.iter()
				.map(|child| parse_node(child, false, selectors))
				.collect::<Result<Vec<_>, _>>()?;
			let object_items = matches!(items[0], DescriptionNode::Object { .. });
			if items
				.iter()
				.any(|item| matches!(item, DescriptionNode::Object { .. }) != object_items)
				|| items.iter().any(|item| matches!(item, DescriptionNode::Array { .. }))
			{
				return Err(node.error("array 只能包含同类标量或 object"));
			}
			Ok(DescriptionNode::Array {
				name,
				description: description(node, require_name)?,
				conditions,
				items,
			})
		}
		_ => Err(node.error("配置说明只接受 object、array、string、integer 或 boolean")),
	}
}

fn description(node: &Element, required: bool) -> Result<String, DslError> {
	if required {
		Ok(node.required("description")?.into())
	} else {
		Ok(node.attr("description").unwrap_or_default().into())
	}
}

fn parse_conditions(
	node: &Element,
	selectors: &BTreeMap<&str, &DescriptionSelector>,
) -> Result<Vec<(String, String)>, DslError> {
	let mut conditions = Vec::new();
	let mut names = BTreeSet::new();
	for condition in node.attr("when").unwrap_or_default().split(',').filter(|s| !s.is_empty()) {
		let Some((name, value)) = condition.split_once('=') else {
			return Err(node.error("when 必须使用 selector=value，并以逗号分隔"));
		};
		if !names.insert(name) || name.is_empty() || value.is_empty() {
			return Err(node.error("when 含有空值或重复选择器"));
		}
		let Some(selector) = selectors.get(name) else {
			return Err(node.error("配置节点引用了不存在的选择器"));
		};
		if !selector.choices.iter().any(|(candidate, _)| candidate == value) {
			return Err(node.error("配置节点引用了不存在的选择值"));
		}
		conditions.push((name.into(), value.into()));
	}
	Ok(conditions)
}

fn combine_conditions(parent: &[(String, String)], child: &[(String, String)]) -> Result<Vec<(String, String)>, DslError> {
	let mut combined = parent.to_vec();
	for (name, value) in child {
		if let Some((_, inherited)) = combined.iter().find(|(candidate, _)| candidate == name) {
			if inherited != value {
				return Err(DslError("配置说明包含永远无法显示的条件节点".into()));
			}
		} else {
			combined.push((name.clone(), value.clone()));
		}
	}
	Ok(combined)
}

fn can_overlap(left: &[(String, String)], right: &[(String, String)]) -> bool {
	!left.iter().any(|(name, value)| {
		right
			.iter()
			.any(|(other_name, other_value)| name == other_name && value != other_value)
	})
}

fn check_duplicate_names(nodes: &[DescriptionNode], parent: &[(String, String)], context: &Element) -> Result<(), DslError> {
	for (index, node) in nodes.iter().enumerate() {
		let conditions = combine_conditions(parent, node.conditions())?;
		for other in &nodes[index + 1..] {
			if node.name() == other.name() && can_overlap(&conditions, &combine_conditions(parent, other.conditions())?) {
				return Err(context.error("同一对象包含可能同时显示的重复 name"));
			}
		}
	}
	Ok(())
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

fn render_yaml(nodes: &[DescriptionNode]) -> Result<Vec<DescriptionLine>, DslError> {
	let mut lines = Vec::new();
	for node in nodes {
		yaml_node(node, 0, &[], false, &mut lines)?;
	}
	Ok(lines)
}

fn yaml_node(
	node: &DescriptionNode,
	depth: usize,
	parent: &[(String, String)],
	array_item: bool,
	lines: &mut Vec<DescriptionLine>,
) -> Result<(), DslError> {
	let conditions = combine_conditions(parent, node.conditions())?;
	let pad = "  ".repeat(depth);
	let prefix = if array_item { "- " } else { "" };
	match node {
		DescriptionNode::Scalar {
			name,
			value,
			description,
			..
		} => {
			let text = match name {
				Some(name) => format!("{pad}{prefix}{}: {}", yaml_key(name), scalar(value)),
				None => format!("{pad}{prefix}{}", scalar(value)),
			};
			lines.push(DescriptionLine {
				text,
				description: description.clone(),
				conditions,
			});
		}
		DescriptionNode::Object {
			name,
			description,
			children,
			..
		} => {
			if let Some(name) = name {
				lines.push(DescriptionLine {
					text: format!("{pad}{prefix}{}:", yaml_key(name)),
					description: description.clone(),
					conditions: conditions.clone(),
				});
			} else {
				lines.push(DescriptionLine {
					text: format!("{pad}-"),
					description: description.clone(),
					conditions: conditions.clone(),
				});
			}
			for child in children {
				yaml_node(child, depth + 1, &conditions, false, lines)?;
			}
		}
		DescriptionNode::Array {
			name,
			description,
			items,
			..
		} => {
			let Some(name) = name else {
				return Err(DslError("不支持嵌套匿名 array".into()));
			};
			lines.push(DescriptionLine {
				text: format!("{pad}{prefix}{}:", yaml_key(name)),
				description: description.clone(),
				conditions: conditions.clone(),
			});
			for item in items {
				yaml_node(item, depth + 1, &conditions, true, lines)?;
			}
		}
	}
	Ok(())
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

fn render_toml(nodes: &[DescriptionNode]) -> Result<Vec<DescriptionLine>, DslError> {
	let mut lines = Vec::new();
	toml_members(nodes, &[], &[], &mut lines)?;
	Ok(lines)
}

fn toml_members(
	nodes: &[DescriptionNode],
	path: &[String],
	parent: &[(String, String)],
	lines: &mut Vec<DescriptionLine>,
) -> Result<(), DslError> {
	for node in nodes {
		let conditions = combine_conditions(parent, node.conditions())?;
		match node {
			DescriptionNode::Scalar {
				name: Some(name),
				value,
				description,
				..
			} => lines.push(DescriptionLine {
				text: format!("{} = {}", toml_key(name), scalar(value)),
				description: description.clone(),
				conditions,
			}),
			DescriptionNode::Array {
				name: Some(name),
				description,
				items,
				..
			} if !matches!(items[0], DescriptionNode::Object { .. }) => {
				lines.push(DescriptionLine {
					text: format!("{} = [", toml_key(name)),
					description: description.clone(),
					conditions: conditions.clone(),
				});
				for item in items {
					let DescriptionNode::Scalar {
						value,
						description,
						conditions: item_conditions,
						..
					} = item
					else {
						return Err(DslError("TOML 标量数组包含非标量元素".into()));
					};
					lines.push(DescriptionLine {
						text: format!("  {},", scalar(value)),
						description: description.clone(),
						conditions: combine_conditions(&conditions, item_conditions)?,
					});
				}
				lines.push(DescriptionLine {
					text: "]".into(),
					description: description.clone(),
					conditions,
				});
			}
			_ => {}
		}
	}
	for node in nodes {
		let conditions = combine_conditions(parent, node.conditions())?;
		match node {
			DescriptionNode::Object {
				name: Some(name),
				description,
				children,
				..
			} => {
				let mut child_path = path.to_vec();
				child_path.push(name.clone());
				lines.push(DescriptionLine {
					text: format!("[{}]", toml_path(&child_path)),
					description: description.clone(),
					conditions: conditions.clone(),
				});
				toml_members(children, &child_path, &conditions, lines)?;
			}
			DescriptionNode::Array {
				name: Some(name),
				description,
				items,
				..
			} if matches!(items[0], DescriptionNode::Object { .. }) => {
				let mut item_path = path.to_vec();
				item_path.push(name.clone());
				for item in items {
					let DescriptionNode::Object {
						description: item_description,
						conditions: item_conditions,
						children,
						..
					} = item
					else {
						return Err(DslError("TOML 对象数组包含非对象元素".into()));
					};
					let item_conditions = combine_conditions(&conditions, item_conditions)?;
					lines.push(DescriptionLine {
						text: format!("[[{}]]", toml_path(&item_path)),
						description: if item_description.is_empty() {
							description.clone()
						} else {
							item_description.clone()
						},
						conditions: item_conditions.clone(),
					});
					toml_members(children, &item_path, &item_conditions, lines)?;
				}
			}
			_ => {}
		}
	}
	Ok(())
}
