# 配置描述 DSL

生成器使用 **Config DSL v4**。`schema/schemas.txt` 注册默认构建内嵌的应用级 schema；当前 `tuic-server.xml` 与 `tuic-client.xml` 分别定义 TUIC 服务端和客户端。品牌、版本、链接、字段、集合、默认值、校验、联动、提示、启动命令及输出结构均在 XML 中。Rust 实现通用注册、XML 解释器、编辑状态、表单视图、随机字节编码和序列化；Svelte 渲染通用控件。两者都不认识服务端、客户端或 TUIC 配置字段。

v4 替换 v3 描述格式，旧 XML 需要迁移；已生成的 TUIC 配置保持兼容。测试目录中的旧版类型和期望输出仅用于 TUIC 回归验证，不编译进应用。

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

Svelte 通过 WASM `Engine` 提交 `set`、`set-row`、`add`、`remove`、`generate` 和 `generate-row` 操作。Rust `Session` 统一处理状态和 XML 联动，再返回已计算可见性、错误及预览的显示快照。前端不接收条件表达式，也不解释 DSL。集合行以独立字符串标识作为 Svelte 的 keyed each 键，业务字段 `id` 不受影响；复制下载另行请求原始导出文本。此界面重构不改变 DSL v4 语法。

## 文档结构

```xml
<config-dsl version="4" target-version="example-1">
  <ui title="示例配置" brand="Example" format-field="encoding">
    <section name="general" label="设置"/>
  </ui>
  <validators>
    <validator name="nonempty" kind="required" message="请填写名称。"/>
  </validators>
  <inputs>
    <field name="encoding" type="enum" default="json" label="输出格式">
      <option value="json" label="JSON"/>
    </field>
    <field name="title" type="string" default="example" label="名称"
           section="general" rule="nonempty"/>
  </inputs>
  <outputs>
    <object name="settings" label="应用配置" filename="settings"
            command="example --config {filename}">
      <string name="title" from="/title"/>
    </object>
  </outputs>
</config-dsl>
```

`inputs`、`outputs` 各必须有一个。`ui`、`config-desc`、`validators`、`conditions`、`values`、`rules`、`effects` 可选，每个区块最多一个。顶层输出数量和名称不限于两个固定角色。

## 配置详解

可选的 `config-desc` 区块为“预览区”提供逐行说明。合并后的页面只有“选择配置区”（表单）和“预览区”两个区域，没有独立的“配置详解”视图或页签；预览区按配置详解的样式显示**表单实时生成**的配置：行号、Prism 逐行高亮、悬浮或键盘聚焦查看 `description`，并叠加生成器的校验结果（可导出状态、错误列表、复制与下载）。`ui/prism.ts` 注册 YAML/TOML/JSON 语法，颜色由 `--tok-*` 变量在明暗主题下定义；高亮只影响展示。

`config-desc` 的 `config` 按 `name` 与顶层输出对应，没有对应说明的输出回退为无说明的纯文本行。生成配置与说明模板按成员名匹配：同名成员互斥时用示例 `value` 选择，记录等动态键（如 `users`、具名 `outbound`）复用对象中唯一的成员作为模板，数组按位置复用元素模板。因此 `config-desc` 的结构应与 `outputs` 保持一致；分支由表单控制，`selector` 不再驱动界面，仅保留用于解析与静态校验。

页面级方案来自 `schema/schemas.txt`，与单份 XML 内的输出和 `config-desc/config` 名称无关。页面可通过 `?schema=<schema-id>` 直接选择应用方案，并保留其他查询参数和片段；`?mode=generate|detail` 兼容读取但不再切换视图。切换 schema 会创建全新会话，不复用上一应用的输入或凭据。

```xml
<config-desc title="配置项详解" description="逐行说明生成的配置。">
  <config name="server" label="服务端配置" filename="server">
    <selector name="backend" label="后端" default="quinn">
      <choice value="quinn" label="Quinn"/>
      <choice value="other" label="其他"/>
    </selector>
    <string name="server" value="[::]:8443" description="UDP 监听地址。"/>
    <object name="backend" description="后端配置。">
      <string name="mode" value="quinn" description="使用 Quinn 后端。" when="backend=quinn"/>
    </object>
  </config>
</config-desc>
```

`config-desc` 至少包含一个 `config`，每个配置至少有一个结构节点。`config` 的 `name`、`label`、`filename` 必填；`filename` 是不含扩展名的安全文件名。`selector` 的 `name`、`label`、`default` 必填，且默认值必须属于其非空 `choice` 列表。

结构节点包括 `object`、`array`、`string`、`integer` 和 `boolean`。对象成员必须声明 `name`；数组元素不能声明 `name`。标量使用 `value` 表达显式类型，所有可见配置项使用 `description` 提供说明。`array` 可以包含同类标量或 `object`，不能混合两种形态。可选 `when` 可用于任意层级，使用逗号分隔的 `selector=value` 条件；子节点继承父节点条件。相同对象中的同名成员只有在选择器条件互斥时才合法。渲染器统一处理 YAML 缩进、TOML table、对象数组和标量数组，XML 不含格式专用空格或标点。

## 页面与输入

`ui` 属性：`title`、`brand` 必填；可选 `mark`、`eyebrow`、`description`、`export-hint`、`mode-field`、`format-field`。后两者引用任意顶层枚举，分别显示为模式按钮和输出格式选择器，不要求字段名为 mode 或 format。格式枚举只能包含 `toml`、`json`、`yaml`；未绑定格式时默认 JSON。

`section` 用 `name` 绑定输入的 `section`，`label` 指定标题，`detail` 指定补充文字，`collapsed="true"` 使用折叠面板，`when` 控制整个分区显示。`notice text="…" when="…"` 可放在分区内或 `ui` 下，后者出现在导出区域。所有文字作为文本呈现，不解析 HTML；分区显示条件不隐式改变输出或校验，应在对应字段和输出声明 `when`。

显式声明分区后，所有普通字段和集合必须绑定到已声明的分区；模式、格式和集合索引使用专用控件。没有分区时根据输入分区名生成通用分区。

`field` 必须有 `name`、`type`、`default`、`label`：

| 类型 | 默认值及控件 |
| --- | --- |
| `string` | 原样字符串；支持 `widget="text|password|number|email"` |
| `boolean` | 仅 `true` / `false`；复选框 |
| `integer` | 非负整数；数字输入 |
| `enum` | 字符串，必须在子元素 `option value="…" label="…"` 中 |

可选 `placeholder`、`hint`、`section`、`when`、`rule`。`when` 引用条件，同时控制字段显示和字段校验；隐藏值保留。可编辑数字通常使用 string + number 控件，让无效输入保留在表单中并显示错误。

`rule` 引用 XML 定义的 validator 名称，不是硬编码的业务规则名。

### 任意集合、索引与随机值

`collection` 包含任意数量的 `field`，支持同样的六种控件，无专用用户或转发类型。

| 属性 | 含义 |
| --- | --- |
| `name`、`initial-items` | 集合名称、初始行数（0–1000） |
| `label`、`section`、`hint` | 行名称、所属分区、说明 |
| `add-label`、`generate-label` | 添加按钮、随机生成按钮的文字 |
| `when` | 集合的显示及校验条件 |
| `min-items` | 最少保留行数，默认 0，不能超过初始行数 |
| `selected-by` | 指向顶层 integer 输入，显示行选择器 |
| `all-when` | 满足时允许全部行编辑和校验；否则仅显示并校验选中行 |
| `select-when` | 行选择器及索引校验的启用条件；缺省始终启用 |

删除选中行后索引归零；删除其前面的行会递减索引。页面行标识保存在状态数据之外，不占用 `id` 或其他业务字段，增删时保留其他行的 DOM 身份。

任意字符串字段可声明 `generator="uuid-v4"` 或 `generator="hex" bytes="24"`。hex 默认 24 字节，允许 1–1024；UUID 固定使用 16 字节并设置版本和 variant 位。浏览器 Crypto API 提供随机字节；生成按钮只更新声明了 generator 的字段。初次载入和新增行也遵循这些声明。XML 默认值不储存生成后的凭据。

## 条件、来源与转换

绝对路径 `/name` 读取顶层输入，相对路径 `name` 读取当前行。只接受单段字段名；不支持脚本、深层路径或通配符。命名值始终在顶层上下文计算。

| 条件 | 行为 |
| --- | --- |
| `all` / `any` | 至少一个子条件；短路与、或 |
| `not` | 一个子条件，取反 |
| `use ref="name"` | 引用命名条件 |
| `eq from="…" value="…"` | 比较标量的文本表示 |
| `truthy from="…"` | 读取布尔值，不接受字符串布尔值 |
| `ip from="…"` | 字符串是否是 IPv4/IPv6 |
| `valid from="…" rule="…"` | 应用声明的 validator |
| `compare op="gte|ne"` | 两个子值元素；非负整数大于等于，或值不等 |

`eq`、`truthy`、`ip`、`valid` 的 `from` 可换成 `ref` 读取命名值。`conditions` 中的 `condition name="…"` 包含一个条件。`values` 中的 `value name="…"` 包含一个值元素：

| 值元素 | 行为 |
| --- | --- |
| `source from="…"` / `source ref="…"` | 读取字段或命名值；支持 `when`、`transform` |
| `coalesce` | 取首个非 null、非空字符串；保留 false 和 0 |
| `endpoint` | 两个子值：主机和整数端口；自动加 IPv6 方括号 |
| `select from="/rows" index="/chosen"` | 根据索引选中行，再计算唯一子值；越界拒绝 |

`transform` 是固定操作名序列：`trim`、`lowercase`、`unbracket`、`integer`、`socket`、`port`。每个操作读取字符串；`integer` 解析十进制非负整数，`socket` 把 IP 端点规范化为用于比较的键，`port` 从 IP 端点提取整数端口。没有隐式类型转换。字符串输出支持 `unit="s|ms"`，为整数值附加单位。

## 校验和字段联动

`validator name="…" kind="…" message="…"` 定义可复用的检查；可指定 `transform`、`min`、`max` 和 `nonblank="true"`（要求字符串去除首尾空白后非空）。

| kind | 检查 |
| --- | --- |
| `required` | 字符串去除首尾空白后非空 |
| `length` | 字符串 UTF-8 字节数或列表项数满足 min/max |
| `integer` | 非负十进制安全解析为 u64，并应用 XML 中的 min/max |
| `optional-integer` | 空字符串通过；非空时按 `integer` 检查 |
| `host` | 可连接的域名或 IP，不是通配监听地址 |
| `socket` / `endpoint` | IP:端口 / 域名或 IP:端口 |
| `email` | 基本邮箱格式 |
| `uuid` | 非零、标准连字符 UUID；需要忽略大小写时显式声明转换 |
| `domain` / `public-domain` | DNS 名称 / 含点的 DNS 名称（语法检查，不查询 DNS） |
| `loopback-socket` | 监听端点的 IP 是回环地址 |

所有协议限制数值和错误文案在 XML 中，例如认证字节上限、毫秒范围。基础检查器不包含 TUIC 字段名。

`rules` 声明跨字段约束：

```xml
<assert key="last" when="range" message="结束数量不能小于起始数量。">
  <compare op="gte">
    <source from="/last" transform="integer"/>
    <source from="/first" transform="integer"/>
  </compare>
</assert>
<unique collection="entries" key="id" message="名称重复。">
  <source from="id" transform="trim lowercase"/>
</unique>
```

`assert` 含一个必须成立的条件；加 `collection` 后逐行检查。`unique` 在可见且参与校验的行中检查一个或多个子值组成的键。`key` 是错误绑定字段，行错误自动形成 `collection.index.field`；`message`、可选 `when` 均在 XML 中。已有同字段错误时保留先前错误；比较前可用 valid 条件排除尚未合法的数字或端点。

`effects` 中的 `<reset on="field" target="other"/>` 表示输入实际改变时，把目标恢复到它的 XML 默认值；不支持脚本，也不递归触发重置。TUIC 的证书切换重置、证书/SNI 关系、重连顺序、凭据唯一性及监听冲突均由这些通用声明表达。

## 输出与脱敏

`outputs` 是任意命名输出的集合。顶层 `object` 可声明 `label`、`filename`（不含扩展名）、`command`。文件扩展名由格式选择决定；命令中的 `{filename}` 替换为完整文件名，只显示、不执行。没有固定输出名或命令前缀。

| 输出元素 | 行为 |
| --- | --- |
| `object` | 对象，子节点 name 唯一 |
| `string` / `boolean` / `integer` | 严格标量类型；输出 integer 为有符号 64 位 |
| `enum options="field"` | 使用顶层枚举的选项校验 |
| `list from="/rows"` | 将每行映射成唯一子输出；支持 where-field + equals 过滤 |
| `list` | 在当前上下文产生一个元素 |
| `record from="/rows" key="field"` | 动态映射键；可声明 key-transform；空键或规范化后重复会报错 |

标量指定一个且只能一个 `from`、`ref`、`value` 或子值元素。所有输出支持 `when`、`secret="true"`。隐藏输出不读取来源；false、0、空字符串、空集合保留。集合加 `omit-empty="true"` 才省略空结果。

脱敏遍历生成后的输出树，不重新读取输入；敏感值替换为 `••••••••`，未声明字段和类型错误拒绝预览。复制、下载始终使用原始输出。JSON 额外转义 Unicode 分隔符；三种格式均有独立解析器往返检查。

## 解析边界与维护

`dsl/xml.rs` 检查 XML 子集和资源上限；`dsl/wire.rs` 使用 quick-xml + Serde 反序列化；`dsl/parser.rs` 与 `dsl/metadata.rs` 检查结构及引用；`dsl/rules.rs` 解释校验和联动；`dsl.rs` 投影、脱敏；`schema.rs` 处理嵌入描述和通用页面状态。

支持单双引号、配对/自闭合标签、注释、五种标准实体和字符引用。不支持 BOM、XML 声明、DTD、外部实体、命名空间、处理指令、CDATA 或混合文本。属性空白按解码结果保留。大小上限 1 MiB，最大元素深度 64，在递归反序列化前检查。未知标签/属性、非法默认值、重复名称、未声明引用、循环条件/值依赖、错误界面绑定和随机生成参数会被拒绝。错误带位置或路径，不回显输入值。

新增目标应用的信息时编辑 XML；新增通用表达能力或控件才修改 Rust，并用 `schema/example.xml` 和通用测试验证。运行 README 中的测试、两种目标的 Clippy、格式化、三格式往返及浏览器回归。TUIC 解析测试只作为独立兼容性检查，生成器构建不依赖相邻 TUIC 仓库。
