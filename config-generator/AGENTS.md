# 配置描述 DSL

生成器使用 **Config DSL v5**。`schema/schemas.txt` 注册默认构建内嵌的应用级 schema；当前 `tuic-server.xml` 与 `tuic-client.xml` 分别定义 TUIC 服务端和客户端。品牌、版本、链接、字段、集合、默认值、校验、联动、提示、启动命令及输出结构均在 XML 中。Rust 实现通用注册、XML 解释器、编辑状态、表单视图、随机字节编码和序列化；Svelte 渲染通用控件。两者都不认识服务端、客户端或 TUIC 配置字段。

v5 把逐行说明并入 `outputs` 节点和枚举选项并移除 `config-desc`，旧 XML 需要迁移；已生成的 TUIC 配置保持兼容。测试目录中的旧版类型和期望输出仅用于 TUIC 回归验证，不编译进应用。

## 更换描述

默认构建嵌入 `schema/schemas.txt` 列出的全部 XML。清单每行是 `schema-id | 显示名称 | XML 文件名`；ID 只允许小写 ASCII 字母、数字和连字符，顺序决定默认项。新增 `wind` 等应用时添加独立 XML 和一行清单项即可。浏览器只接受已注册 ID，不读取 URL 指定的文件或网络配置。可使用 `CONFIG_SCHEMA` 选择一份独立 XML 进行复用测试，路径相对 `config-generator/`，也支持绝对路径。库调用不依赖嵌入文件：`Document::parse(xml)` → `State::new(&document)` → `model::build_configs(&document, &state.data)`。

```powershell
$env:CONFIG_SCHEMA = 'schema/example.xml'
try {
    npm run build --prefix config-generator -- --outDir ../.cache/generic-site
    uvx python tests/config-generator/run-browser.py --directory .cache/generic-site --script tests/config-generator/browser-generic.mjs
} finally {
    Remove-Item Env:CONFIG_SCHEMA
    npm run wasm --prefix config-generator
}
```

`schema/example.xml` 是不含 TUIC 字段的任务清单示例，具有不同的品牌、输入名、集合、三个输出和导出命令。相同 Rust/Svelte 代码直接呈现它，不需要增加分支或修改 HTML。环境变量变更和选中文件变更均会触发 Cargo 重新嵌入；这是构建时切换，网页不下载外部 XML、不读取网络配置。上面的 `--outDir` 相对 `config-generator/`，保留默认 `dist/` 和组合站点产物；最后重新生成默认 `pkg/`，避免后续 Vite 开发会话继续使用示例 WASM。

Svelte 通过 WASM `Engine` 提交 `set`、`set-row`、`add`、`remove`、`generate` 和 `generate-row` 操作。Rust `Session` 统一处理状态和 XML 联动，再返回已计算可见性、错误及预览的显示快照。前端不接收条件表达式，也不解释 DSL。集合行以独立字符串标识作为 Svelte 的 keyed each 键，业务字段 `id` 不受影响；复制下载另行请求原始导出文本。校验不通过时预览不再隐藏：无效输入值在预览中显示为 `<placeholder>`，并保留警告；导出仍走严格投影，校验通过前复制和下载保持禁用。此界面重构不改变 DSL v5 语法。

前端采用分区目录、卡片表单和固定预览组成的响应式工作台，小屏幕下按顺序排列。目录操作会展开目标分区并移动键盘焦点；错误列表会展开字段所在分区并定位输入框。`AppHeader` 管理方案 URL 和主题入口，`SectionNavigation` 管理分区跳转，`ExportActions` 管理复制下载；`Controller` 统一管理 WASM 会话与预览密码显示状态，切换方案时重置显示状态。主题和展开状态仅存在于当前页面，不持久化。

## DSL 概要

描述文件为 `config-dsl version="5"`。顶层区块：`ui`、`validators`、`inputs`（必需）、`conditions`、`values`、`outputs`（必需）、`rules`、`effects`；每个区块至多一个，顺序无关。逐行预览说明直接声明在 `outputs` 节点和枚举 `option` 的 `description` 上，是展示元数据，不参与投影、校验、脱敏或导出。

完整的元素、属性、默认值、约束与示例见 [`../wind/docs/config-dsl.md`](../wind/docs/config-dsl.md)（英文版 [`../wind/docs/specs/config-dsl.md`](../wind/docs/specs/config-dsl.md)，发布地址 <https://rust-proxy.github.io/wind/config-dsl/>）。

维护规则：

- 产品信息只写进 XML；只有新增通用表达能力或控件才修改 Rust，并用 `schema/example.xml` 和通用测试验证。
- `description` 必须与 `outputs` 结构一致；没有说明的输出行不显示提示。
- 不引入脚本、网络查询或可执行表达式；XML 子集与资源上限由 `dsl/xml.rs` 检查。

## 解析边界与维护

`dsl/xml.rs` 检查 XML 子集和资源上限；`dsl/wire.rs` 使用 quick-xml + Serde 反序列化；`dsl/parser.rs` 与 `dsl/metadata.rs` 检查结构及引用；`dsl/rules.rs` 解释校验和联动；`dsl.rs` 投影、脱敏；`dsl/preview.rs` 沿输出结构生成带说明的预览行；`schema.rs` 处理嵌入描述和通用页面状态。

支持单双引号、配对/自闭合标签、注释、五种标准实体和字符引用。不支持 BOM、XML 声明、DTD、外部实体、命名空间、处理指令、CDATA 或混合文本。属性空白按解码结果保留。大小上限 1 MiB，最大元素深度 64，在递归反序列化前检查。未知标签/属性、非法默认值、重复名称、未声明引用、循环条件/值依赖、错误界面绑定和随机生成参数会被拒绝。错误带位置或路径，不回显输入值。

新增目标应用的信息时编辑 XML；新增通用表达能力或控件才修改 Rust，并用 `schema/example.xml` 和通用测试验证。运行 README 中的测试、两种目标的 Clippy、格式化、三格式往返及浏览器回归。TUIC 解析测试只作为独立兼容性检查，生成器构建不依赖相邻 TUIC 仓库。
