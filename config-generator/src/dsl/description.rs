use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

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
	pub lines: Vec<DescriptionLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescriptionSelector {
	pub name: String,
	pub label: String,
	pub default: String,
	pub choices: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescriptionLine {
	pub yaml: String,
	pub description: String,
	pub conditions: Vec<(String, String)>,
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
		let mut selectors = Vec::new();
		let mut selector_names = BTreeSet::new();
		let mut lines = Vec::new();
		for child in &config.children {
			match child.tag.as_str() {
				"selector" => {
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
				"line" => {
					attrs(child, &["yaml", "description", "when"])?;
					if !child.children.is_empty() {
						return Err(child.error("line 不接受子元素"));
					}
					lines.push(DescriptionLine {
						yaml: child.required("yaml")?.into(),
						description: child.required("description")?.into(),
						conditions: parse_conditions(child)?,
					});
				}
				_ => return Err(child.error("config 只接受 selector 或 line")),
			}
		}
		if lines.is_empty() {
			return Err(config.error("配置说明至少需要一行 YAML"));
		}
		let selector_map: BTreeMap<_, _> = selectors.iter().map(|s| (s.name.as_str(), s)).collect();
		for line in &lines {
			for (selector, value) in &line.conditions {
				let Some(selector) = selector_map.get(selector.as_str()) else {
					return Err(config.error("YAML 行引用了不存在的选择器"));
				};
				if !selector.choices.iter().any(|(candidate, _)| candidate == value) {
					return Err(config.error("YAML 行引用了不存在的选择值"));
				}
			}
		}
		configs.push(DescriptionConfig {
			name: name.into(),
			label: config.required("label")?.into(),
			filename: config.required("filename")?.into(),
			selectors,
			lines,
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

fn parse_conditions(node: &Element) -> Result<Vec<(String, String)>, DslError> {
	let mut conditions = Vec::new();
	let mut names = BTreeSet::new();
	for condition in node.attr("when").unwrap_or_default().split(',').filter(|s| !s.is_empty()) {
		let Some((name, value)) = condition.split_once('=') else {
			return Err(node.error("when 必须使用 selector=value，并以逗号分隔"));
		};
		if !names.insert(name) || name.is_empty() || value.is_empty() {
			return Err(node.error("when 含有空值或重复选择器"));
		}
		conditions.push((name.into(), value.into()));
	}
	Ok(conditions)
}
