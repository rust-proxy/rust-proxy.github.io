//! Built-in framework strings. Schemas may override any id by declaring the same name in their
//! `<strings>` pool; otherwise these catalog entries are used. Kept deliberately small and free
//! of product identifiers.
const CATALOG: &[(&str, &str, &str)] = &[
	("builtin.ui.title", "zh-CN", "配置文件编辑器"),
	("builtin.ui.title", "en", "Configuration editor"),
	("builtin.ui.brand", "zh-CN", "配置工具"),
	("builtin.ui.brand", "en", "Config tools"),
	("builtin.ui.section", "zh-CN", "输入"),
	("builtin.ui.section", "en", "Inputs"),
	("builtin.add", "zh-CN", "添加"),
	("builtin.add", "en", "Add"),
	("builtin.generate", "zh-CN", "生成随机值"),
	("builtin.generate", "en", "Generate value"),
	("builtin.error.select", "zh-CN", "请选择有效选项。"),
	("builtin.error.select", "en", "Select a valid option."),
	("builtin.error.collection", "zh-CN", "输入集合必须是列表。"),
	("builtin.error.collection", "en", "The input collection must be a list."),
	("builtin.error.collection-min", "zh-CN", "{label}至少需要 {count} 项。"),
	(
		"builtin.error.collection-min",
		"en",
		"{label} requires at least {count} items.",
	),
	("builtin.error.selection", "zh-CN", "请选择有效项目。"),
	("builtin.error.selection", "en", "Select a valid item."),
];

/// Resolves a built-in id for a locale, falling back to the schema default locale, then `en`.
pub fn builtin(id: &str, locale: &str, default_locale: &str) -> Option<&'static str> {
	let lookup = |want: &str| {
		CATALOG
			.iter()
			.find(|(name, tag, _)| *name == id && *tag == want)
			.map(|(_, _, text)| *text)
	};
	lookup(locale).or_else(|| lookup(default_locale)).or_else(|| lookup("en"))
}

/// Substitutes `{label}` and `{count}` placeholders in a built-in message.
pub fn substitute(template: &str, label: &str, count: usize) -> String {
	template.replace("{label}", label).replace("{count}", &count.to_string())
}
