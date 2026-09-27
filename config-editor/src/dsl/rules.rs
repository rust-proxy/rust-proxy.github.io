use super::*;
use crate::{
	messages::{self, builtin},
	validation::{self, Errors},
};

impl Collection {
	pub fn row_visible(&self, doc: &Document, root: &Value, index: usize) -> Result<bool, DslError> {
		if !self.visible_in(doc, root)? {
			return Ok(false);
		}
		if let Some(selected) = &self.selected_by {
			let all = self
				.all_when
				.as_ref()
				.map(|name| doc.condition(name, root, root))
				.transpose()?
				.unwrap_or(true);
			if !all {
				return Ok(root[selected].as_u64() == Some(index as u64));
			}
		}
		Ok(true)
	}
	pub fn editable(&self, doc: &Document, root: &Value) -> bool {
		self.all_when
			.as_ref()
			.is_none_or(|name| doc.condition(name, root, root).unwrap_or(false))
	}
}
impl Document {
	pub fn shown(&self, when: &Option<String>, root: &Value) -> bool {
		when.as_ref()
			.is_none_or(|name| self.condition(name, root, root).unwrap_or(false))
	}
	pub fn write_field(&self, root: &mut Value, key: &str, value: &str) -> bool {
		let Some(field) = self.fields.iter().find(|f| f.key == key) else {
			return false;
		};
		let before = root.get(key).cloned();
		if !field.write_value(root, value) {
			return false;
		}
		if before.as_ref() != root.get(key) {
			for (_, target) in self.resets.iter().filter(|(on, _)| on == key) {
				if let Some(f) = self.fields.iter().find(|f| &f.key == target) {
					root[target] = f.default.clone();
				}
			}
		}
		true
	}
	pub(super) fn check_validator(&self, name: &str, value: &Value) -> Result<bool, DslError> {
		let n = self.validators.get(name).ok_or_else(|| DslError("未声明的校验规则".into()))?;
		let value = transform(n, value.clone(), n.attr("transform"))?;
		if n.attr("nonblank") == Some("true") && value.as_str().is_none_or(|v| v.trim().is_empty()) {
			return Ok(false);
		}
		let text = scalar_text(&value);
		let range = |v: u64| {
			n.attr("min").and_then(|s| s.parse::<u64>().ok()).is_none_or(|min| v >= min)
				&& n.attr("max").and_then(|s| s.parse::<u64>().ok()).is_none_or(|max| v <= max)
		};
		Ok(match n.required("kind")? {
			"required" => value.as_str().is_some_and(|v| !v.trim().is_empty()),
			"length" => match &value {
				Value::String(s) => range(s.len() as u64),
				Value::Array(a) => range(a.len() as u64),
				_ => false,
			},
			"integer" => !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) && text.parse::<u64>().is_ok_and(range),
			"optional-integer" => {
				text.is_empty() || (text.bytes().all(|b| b.is_ascii_digit()) && text.parse::<u64>().is_ok_and(range))
			}
			"host" => validation::host_rule(&text).is_none(),
			"socket" => validation::endpoint(&text, true).is_some(),
			"endpoint" => validation::endpoint(&text, false).is_some(),
			"email" => validation::email_rule(&text).is_none(),
			"uuid" => uuid::Uuid::parse_str(&text).is_ok_and(|v| !v.is_nil() && v.hyphenated().to_string() == text),
			"domain" => validation::is_domain(&text),
			"public-domain" => validation::is_domain(&text) && text.contains('.'),
			"loopback-socket" => validation::endpoint(&text, true)
				.is_some_and(|(host, _)| host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())),
			_ => return Err(n.error("未知校验类型")),
		})
	}
	fn validate_fields(
		&self,
		fields: &[InputField],
		root: &Value,
		row: &Value,
		prefix: &str,
		locale: &str,
		errors: &mut Errors,
	) {
		for field in fields {
			let key = format!("{prefix}{}", field.key);
			let result = (|| {
				if !field.visible_in(self, root, row)? {
					return Ok(None);
				}
				let value = row.get(&field.key).ok_or_else(|| DslError("缺少输入字段".into()))?;
				if !matches!(
					(&field.default, value),
					(Value::String(_), Value::String(_))
						| (Value::Bool(_), Value::Bool(_))
						| (Value::Number(_), Value::Number(_))
				) || (field.default.is_number() && value.as_u64().is_none())
				{
					return Err(DslError("输入字段类型不匹配".into()));
				}
				if field.kind == InputKind::Select && !field.options.iter().any(|(k, _)| Some(k.as_str()) == value.as_str()) {
					return Ok(Some(
						builtin("builtin.error.select", locale, &self.default_locale)
							.unwrap_or_default()
							.to_owned(),
					));
				}
				if !field.rule.is_empty() && !self.check_validator(&field.rule, value)? {
					let message = self.validators[&field.rule].required("message")?;
					return Ok(Some(self.text(&Text::from_attr(message), locale)));
				}
				Ok(None)
			})();
			match result {
				Ok(Some(message)) => {
					errors.insert(key, message);
				}
				Err(e) => {
					errors.insert(key, e.to_string());
				}
				_ => {}
			}
		}
	}
	pub fn validate(&self, root: &Value) -> Errors {
		self.validate_in(root, &self.default_locale)
	}
	pub fn validate_in(&self, root: &Value, locale: &str) -> Errors {
		let mut errors = Errors::new();
		self.validate_fields(&self.fields, root, root, "", locale, &mut errors);
		for c in &self.collections {
			match c.visible_in(self, root) {
				Ok(false) => continue,
				Err(e) => {
					errors.insert(c.name.clone(), e.to_string());
					continue;
				}
				Ok(true) => {}
			}
			let Some(rows) = root.get(&c.name).and_then(Value::as_array) else {
				errors.insert(
					c.name.clone(),
					builtin("builtin.error.collection", locale, &self.default_locale)
						.unwrap_or_default()
						.to_owned(),
				);
				continue;
			};
			if rows.len() < c.min_items {
				let template = builtin("builtin.error.collection-min", locale, &self.default_locale).unwrap_or_default();
				let label = self.text(&c.label, locale);
				errors.insert(c.name.clone(), messages::substitute(template, &label, c.min_items));
			}
			if let Some(key) = &c.selected_by
				&& self.shown(&c.select_when, root)
				&& root[key].as_u64().is_none_or(|i| i >= rows.len() as u64)
			{
				errors.insert(
					key.clone(),
					builtin("builtin.error.selection", locale, &self.default_locale)
						.unwrap_or_default()
						.to_owned(),
				);
			}
			for (i, row) in rows.iter().enumerate() {
				if c.row_visible(self, root, i).unwrap_or(false) {
					self.validate_fields(&c.fields, root, row, &format!("{}.{i}.", c.name), locale, &mut errors);
				}
			}
		}
		for rule in &self.rules {
			let collection = rule
				.attr("collection")
				.and_then(|name| self.collections.iter().find(|c| c.name == name));
			let single = vec![root.clone()];
			let rows = if let Some(c) = collection {
				root.get(&c.name).and_then(Value::as_array).unwrap_or(&single)
			} else {
				&single
			};
			let mut seen = BTreeSet::new();
			for (i, row) in rows.iter().enumerate() {
				if collection.is_some_and(|c| !c.row_visible(self, root, i).unwrap_or(false)) {
					continue;
				}
				let key = if let Some(c) = collection {
					format!("{}.{i}.{}", c.name, rule.attr("key").unwrap_or_default())
				} else {
					rule.attr("key").unwrap_or_default().to_owned()
				};
				if errors.contains_key(&key) {
					continue;
				}
				let result = (|| {
					if !self.visible(rule, root, row)? {
						return Ok(true);
					}
					if rule.tag == "assert" {
						self.test(rule.single()?, root, row)
					} else {
						let values = rule
							.children
							.iter()
							.map(|n| self.eval(n, root, row))
							.collect::<Result<Vec<_>, _>>()?;
						Ok(seen.insert(Value::Array(values).to_string()))
					}
				})();
				match result {
					Ok(true) => {}
					Ok(false) => {
						let message = Text::from_attr(rule.attr("message").unwrap_or_default());
						errors.insert(key, self.text(&message, locale));
					}
					Err(e) => {
						errors.insert(key, e.to_string());
					}
				}
			}
		}
		errors
	}
}
