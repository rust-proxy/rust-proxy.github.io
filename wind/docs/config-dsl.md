Draft: config-dsl-02
Category: Experimental / Specification Draft
Date: 2026 年 9 月
Language version: 5

# Config DSL：静态配置描述

英文版本：[English](specs/config-dsl.md)

## 本备忘录状态

本文是仓库级标准的提案，不是已采纳的 Wind API 或互联网标准。它描述 TUIC 配置生成器
使用的静态 XML 方言，供其他配置生成器参考。Wind 目前尚未实现此 DSL。草案修订号
`config-dsl-02` 和语言属性 `version="5"` 表示不同的版本。

本文聚焦 XML 文档结构与字段用途。正文第 1–13 节是规范性内容，第 15 节列出规范性引用；
第 14 节和附录 A 为说明性内容。中英文版本使用对应的章节编号，并同步维护。

## 摘要

Config DSL 用静态 XML 描述输入、默认值、适用条件、输出结构、固定转换和敏感值脱敏。
消费者据此生成表单、校验输入并投影结构化配置；TOML、JSON 和 YAML 是下游编码。描述
文件不包含可执行代码。

## 1. 范围与术语

大写的 **MUST**、**MUST NOT**、**SHOULD**、**SHOULD NOT**、**MAY** 分别表示必须、
不得、应该、不应该、可以，采用 BCP 14 的要求级别
（[RFC 2119](https://www.rfc-editor.org/rfc/rfc2119)、
[RFC 8174](https://www.rfc-editor.org/rfc/rfc8174)）。小写用法是普通叙述。

| 术语 | 含义 |
| --- | --- |
| 描述（Description） | 独立于一次输入会话的 DSL 文档 |
| 消费者（Consumer） | 解析、校验并投影描述的实现 |
| 宿主（Host） | 提供编辑、目标专用校验和导出的应用 |
| 根（Root） | 包含顶层输入字段和集合的对象 |
| 行（Row） | 当前输入对象；初始为根，在集合映射或选择时被替换 |
| 投影（Projection） | 从一个输入快照生成的结构化输出 |

DSL 不替代目标应用的配置解析器或线路协议。导入、迁移、包含文件、任意函数、赋值、
递归、网络查询和可执行表达式不属于版本 5。消费者 MUST NOT 将字符串作为 Rust、
JavaScript、Shell、模板或其他编程语言求值。Serde、quick-xml 和 Rust 是实现选择，
不是语言要求。

## 2. 词法格式

### 2.1. 编码与结构语法

文档 MUST 使用无 BOM 的 UTF-8。允许的字符为 U+0009、U+000A、U+000D、
U+0020–U+D7FF、U+E000–U+FFFD 和 U+10000–U+10FFFF，即
[XML 1.0](https://www.w3.org/TR/xml/#charsets) 的 `Char` 范围。
本语言是 XML 变种，不是通用 XML 处理器；不引入 XML 的其他处理规则。

名称区分大小写；开始与结束标签、属性值的两端引号必须匹配。重复属性、元素之间的
非空白文本及根元素后的额外内容 MUST 被拒绝。元素间允许注释，但注释内部 MUST NOT
包含 `--`。XML 声明、DTD、实体声明、命名空间、处理指令、CDATA 和混合内容 MUST
被拒绝。

以下 PEG 使用 `~` 表示顺序、`|` 表示有序选择、`*`/`+` 表示重复、`!` 表示否定前瞻、
`ANY` 表示一个允许的字符。标签匹配、实体合法性和语义约束在此语法之外另行检查。

```text
Document  <- SOI ~ Gap* ~ Element ~ Gap* ~ EOI
Gap       <- Space | Comment
Space     <- " " | "\t" | "\r" | "\n"
Comment   <- "<!--" ~ (!"--" ~ ANY)* ~ "-->"
Element   <- Empty | Paired
Empty     <- "<" ~ XmlName ~ (Space+ ~ Attribute)* ~ Space* ~ "/>"
Paired    <- "<" ~ XmlName ~ (Space+ ~ Attribute)* ~ Space* ~ ">"
             ~ Gap* ~ (Element ~ Gap*)* ~ "</" ~ XmlName ~ Space* ~ ">"
XmlName   <- [A-Za-z_] ~ [A-Za-z0-9_-]*
Attribute <- XmlName ~ Space* ~ "=" ~ Space* ~ Quoted
Quoted    <- '"' ~ (!('"' | "<") ~ ANY)* ~ '"'
           | "'" ~ (!("'" | "<") ~ ANY)* ~ "'"
```

### 2.2. 属性解码与字面量语法

命名实体仅有 `&amp;`、`&lt;`、`&gt;`、`&quot;` 和 `&apos;`。十进制 `&#DIGITS;`
和十六进制 `&#xHEXDIGITS;` 引用 MUST 至少有一位数字，并指向允许的字符。
符号、未知实体、缺失分号、代理码点及越界码点 MUST 被拒绝。解码只进行一次：
`&amp;lt;` 得到文本 `&lt;`，而不是 `<`。

解码后的空白，包括字面的制表符与换行序列，MUST 保留。消费者 MUST NOT 隐式执行
XML 属性空白规范化、换行改写、Unicode 规范化或裁剪。通用 XML 库可能需要适配。

| 实体解码后的形式 | 语法 |
| --- | --- |
| 标识符 | `[A-Za-z_][A-Za-z0-9_-]*` |
| 布尔值 | 仅 `true` 或 `false` |
| 无符号整数 | `[0-9]+`，十进制 |
| 有符号整数 | `-?[0-9]+`，十进制 |

允许前导零，不允许正号、小数或指数形式及两侧空白。数值 `-0` 等于零。
标签、普通字符串和枚举选项值不必是标识符。

## 3. 文档结构

根 MUST 为 `config-dsl`，且仅具有必需的 `version="5"` 和 `target-version` 属性。
`target-version` 是不透明元数据，不是语言选择器。

根包含以下顶层区块。未知或重复区块 MUST 被拒绝；区块顺序无关，除 `inputs` 和
`outputs` 外均可省略，每个区块至多一个。除 `ui` 外，这些区块 MUST NOT 有属性。

| 区块 | 必需 | 作用 |
| --- | --- | --- |
| `ui` | 否 | 品牌、分区、提示等界面元数据 |
| `validators` | 否 | 可复用的字段校验器 |
| `inputs` | 是 | 顶层输入字段与集合 |
| `conditions` | 否 | 命名条件 |
| `values` | 否 | 命名值 |
| `outputs` | 是 | 输出结构与导出元数据 |
| `rules` | 否 | 跨字段约束 |
| `effects` | 否 | 字段联动重置 |

## 4. 界面 `ui`

`ui` 描述页面品牌、分区和提示。全部属性可选，但 `title`、`brand` 在参考实现中必需。

| 属性 | 含义 |
| --- | --- |
| `title` | 页面标题 |
| `brand` | 品牌名称 |
| `mark` | 品牌标记字符 |
| `eyebrow` | 标题上方的小字 |
| `description` | 标题下的说明 |
| `export-hint` | 导出区域提示 |
| `mode-field` | 引用任意顶层 `enum` 字段，渲染为模式按钮 |
| `format-field` | 引用任意顶层 `enum` 字段，渲染为输出格式选择器 |

`mode-field`、`format-field` 引用的字段 MUST 是顶层 `enum`。`format-field` 的选项值
只能是 `json`、`toml`、`yaml`；未绑定格式时默认 JSON。

`ui` 的子元素为 `section` 和 `notice`。

`section` 声明一个界面分区：

| 属性 | 必需 | 含义 |
| --- | --- | --- |
| `name` | 是 | 分区标识，绑定输入的 `section` |
| `label` | 是 | 分区标题 |
| `detail` | 否 | 标题补充文字 |
| `collapsed` | 否 | `true` 使用折叠面板，默认 `false` |
| `when` | 否 | 命名条件，控制整个分区显示 |

`notice` 显示纯文本提示：

| 属性 | 必需 | 含义 |
| --- | --- | --- |
| `text` | 是 | 提示文字，作为文本呈现，不解析 HTML |
| `when` | 否 | 命名条件，控制显示 |

`notice` 放在 `ui` 下时出现在导出区域，放在 `section` 内时出现在该分区。没有声明
任何 `section` 时，消费者根据输入的 `section` 生成通用分区；显式声明分区后，所有
普通字段和集合 MUST 绑定到已声明的分区。

## 5. 输入 `inputs`

`inputs` 的子元素为 `field` 和 `collection`。顶层字段与集合共享同一命名空间，名称
MUST 唯一。

### 5.1. 字段 `field`

| 属性 | 必需 | 含义 |
| --- | --- | --- |
| `name` | 是 | 输入标识符 |
| `type` | 是 | `string`、`boolean`、`integer` 或 `enum` |
| `default` | 是 | 按 `type` 解码的初始值 |
| `label` | 是 | 纯文本标签 |
| `placeholder` | 否 | 占位文字 |
| `hint` | 否 | 字段说明 |
| `section` | 否 | 所属界面分区 |
| `widget` | 否 | `text`、`password`、`number` 或 `email`；仅 `string` |
| `when` | 否 | 命名条件，同时控制显示与校验 |
| `rule` | 否 | 引用的校验器名称 |
| `generator` | 否 | `uuid-v4` 或 `hex`；仅 `string` |
| `bytes` | 否 | 仅 `hex`，随机字节数 1–1024，默认 24 |

| 类型 | 默认值 | 控件 |
| --- | --- | --- |
| `string` | 原样字符串 | 文本框；由 `widget` 决定具体控件 |
| `boolean` | 仅 `true` / `false` | 复选框 |
| `integer` | 非负整数 | 数字输入 |
| `enum` | 必须属于子元素 `option` | 下拉选择 |

`enum` MUST 含一个或多个 `option` 子元素；非枚举字段 MUST NOT 有子元素。

`option`：

| 属性 | 必需 | 含义 |
| --- | --- | --- |
| `value` | 是 | 枚举值，同一字段内唯一 |
| `label` | 是 | 选项文字 |
| `description` | 否 | 该值的预览说明，见第 11.4 节 |

`widget` 不改变数据类型：`type="string" widget="number"` 保留可编辑文本。密码控件
不隐含输出脱敏；脱敏由输出的 `secret` 声明。

`generator` 在初次载入、新增行和生成按钮触发时产生随机值：`uuid-v4` 固定 16 字节
并设置版本和 variant 位；`hex` 使用 `bytes` 指定的字节数。XML 默认值不储存生成后的
凭据。

为假的 `when` 将字段排除在显示和普通校验之外，但 MUST NOT 清除其存储值。适用条件
不是访问控制，也不禁止输出读取该字段。

### 5.2. 集合 `collection`

`collection` 声明一组可增删的同构行，行字段复用第 5.1 节的 `field` 结构；行内不渲染
`section`、`hint` 等布局属性。

| 属性 | 必需 | 含义 |
| --- | --- | --- |
| `name` | 是 | 集合标识 |
| `initial-items` | 是 | 初始行数，0–1000 |
| `label` | 否 | 行名称，默认取 `name` |
| `section` | 否 | 所属界面分区 |
| `hint` | 否 | 集合说明 |
| `add-label` | 否 | 添加按钮文字，默认“添加” |
| `generate-label` | 否 | 随机生成按钮文字，默认“生成随机值” |
| `min-items` | 否 | 最少保留行数，0–1000，不得超过 `initial-items` |
| `selected-by` | 否 | 指向顶层 `integer` 字段，显示行选择器 |
| `all-when` | 否 | 满足时允许全部行编辑与校验；否则仅显示并校验选中行 |
| `select-when` | 否 | 行选择器及索引校验的启用条件；缺省始终启用 |
| `when` | 否 | 集合的显示与校验条件 |

`selected-by` MUST 引用顶层 `integer` 字段，且每个字段至多被一个集合使用；`all-when`、
`select-when` 仅在声明 `selected-by` 时可用。集合 `when` 在根求值；字段 `when` 在
每个适用行中求值。删除选中行后索引归零；删除其前面的行会递减索引。稳定 UI 行标识、
选择控件、凭据生成和删除行为属于宿主，不自动导出。

## 6. 条件 `conditions`

`conditions` 包含 `condition name="ID"`，每个定义恰好有一个条件子元素。条件名 MUST
唯一。条件元素不能具有 `name`、`when` 或转换：

| 元素 | 属性 | 子元素与结果 |
| --- | --- | --- |
| `all` / `any` | 无 | 一个或多个条件；全部/任意为真 |
| `not` | 无 | 恰好一个条件；取反 |
| `use` | 必需 `ref` | 无子元素；引用命名条件 |
| `eq` | `from`/`ref` 恰选一个，必需 `value` | 无子元素；比较标量文本与字面量 |
| `truthy` | `from`/`ref` 恰选一个 | 无子元素；要求并返回布尔值 |
| `ip` | `from`/`ref` 恰选一个 | 无子元素；检查 IPv4/IPv6 字面地址 |
| `valid` | `from`/`ref` 恰选一个，必需 `rule` | 无子元素；应用声明的校验器 |
| `compare` | 必需 `op="ne|gte"` | 恰好两个值元素；非负整数比较或值不等 |

`all`/`any` MUST 从左向右短路求值，遇到错误时传播错误而不是当作假。`eq` 比较标量的
文本表示：字符串保留，布尔值表示为 `true`/`false`，整数使用规范十进制；字符串 `"01"`
不等于 `value="1"`，而整数 1 相等。`truthy` 不转换数字或字符串。`ip` 不裁剪、不移除
方括号、不查询 DNS。

任意节点（字段、集合、输出、规则等）的 `when` 属性 MUST 引用已声明的条件；未声明的
引用 MUST 被拒绝。

## 7. 值 `values`

`values` 包含 `value name="ID"`，每个定义恰好有一个值元素。值名 MUST 唯一，并与条件
名分属不同命名空间。所有值元素接受可选 `when`，在任何来源或子元素求值前检查；为假
时产生缺失值。

| 元素 | `when` 之外的属性 | 子元素 |
| --- | --- | --- |
| `source` | `from`/`ref` 恰选一个，可选 `transform` | 无 |
| `coalesce` | 无 | 一个或多个值元素 |
| `endpoint` | 无 | 恰好两个值元素：主机与端口 |
| `select` | 必需 `from`、`index` | 恰好一个值元素 |

`coalesce` 返回首个既非缺失又非空字符串的结果，保留假、零、空列表和空对象；没有
候选时返回空字符串。`endpoint` 要求主机字符串和 1–65535 的整数端口，主机为无括号
IPv6 时产生 `[host]:port`，否则产生 `host:port`。`select` 在调用者上下文读取列表与
从零开始的非负整数索引，再用选定行求值子元素；缺失数据、类型错误或索引越界 MUST
被拒绝。

### 7.1. 固定转换 `transform`

`transform` 和 `key-transform` 存在时，包含由空白分隔的非空操作序列；缺省表示不
转换。每个操作均要求字符串输入：

| 操作 | 结果 |
| --- | --- |
| `trim` | 移除首尾空白 |
| `lowercase` | 与语言环境无关的小写转换 |
| `unbracket` | 同时存在前导 `[` 和末尾 `]` 时移除一对，否则保留文本 |
| `integer` | 解析十进制非负整数 |
| `socket` | 把 IP 端点规范化为用于比较的键 |
| `port` | 从 IP 端点提取整数端口 |

没有隐式类型转换；`integer` 之后执行字符串操作会因中间值为数字而失败。`unit` 仅用于
`string` 输出，值 MUST 为 `s` 或 `ms`：转换后要求整数，并在其规范十进制表示后追加
后缀，不执行缩放。

## 8. 校验器 `validators`

`validators` 包含 `validator`，定义可复用的字段检查：

| 属性 | 必需 | 含义 |
| --- | --- | --- |
| `name` | 是 | 校验器标识，名称唯一 |
| `kind` | 是 | 检查类型，见下表 |
| `message` | 是 | 失败时的错误文案 |
| `min` / `max` | 否 | 非负整数范围 |
| `transform` | 否 | 校验前应用的转换 |
| `nonblank` | 否 | `true` 时要求字符串去除首尾空白后非空 |

| kind | 检查 |
| --- | --- |
| `required` | 字符串去除首尾空白后非空 |
| `length` | 字符串 UTF-8 字节数或列表项数满足 min/max |
| `integer` | 非负十进制安全解析为 u64，并应用 min/max |
| `optional-integer` | 空字符串通过；非空时按 `integer` 检查 |
| `host` | 可连接的域名或 IP，不是通配监听地址 |
| `socket` | IP:端口 |
| `endpoint` | 域名或 IP:端口 |
| `email` | 基本邮箱格式 |
| `uuid` | 非零、标准连字符 UUID |
| `domain` | DNS 名称 |
| `public-domain` | 含点的 DNS 名称 |
| `loopback-socket` | 监听端点的 IP 是回环地址 |

字段的 `rule` MUST 引用已声明的校验器；未声明的规则 MUST 被拒绝，不能静默跳过。
这些检查只做语法校验，不查询 DNS，也不证明可达性、邮件可投递性或 TLS 信任。

## 9. 跨字段规则 `rules`

`rules` 包含 `assert` 或 `unique`：

| 属性 | 必需 | 含义 |
| --- | --- | --- |
| `key` | 是 | 错误绑定字段 |
| `message` | 是 | 失败文案 |
| `when` | 否 | 命名条件 |
| `collection` | 否 | 逐行检查的集合 |

- `assert` 恰好有一个条件子元素；条件为假时报告错误。
- `unique` 有一个或多个值元素；对可见且参与校验的行，检查这些值组成的键是否唯一。

`key` 是错误绑定字段：顶层错误使用字段名，行错误自动形成 `collection.index.field`。
已有同字段错误时保留先前错误。比较前可用 `valid` 条件排除尚未合法的数字或端点。

## 10. 联动 `effects`

`effects` 包含 `reset`：

| 属性 | 必需 | 含义 |
| --- | --- | --- |
| `on` | 是 | 触发字段 |
| `target` | 是 | 被重置字段 |

当 `on` 字段的实际值发生变化时，把 `target` 恢复到它的 XML 默认值。不支持脚本，
也不递归触发重置。`on`、`target` MUST 引用已声明的顶层字段。

## 11. 输出 `outputs`

`outputs` 无属性，其子元素是顶层输出；通常为具名 `object`，对应一个导出文件。

| 输出元素 | 作用 |
| --- | --- |
| `object` | 对象；子节点名称唯一 |
| `list` | 列表；恰好一个未命名子元素作为项模板 |
| `record` | 动态键映射；恰好一个未命名子元素 |
| `string` / `boolean` / `integer` | 严格标量类型 |
| `enum` | 引用顶层枚举的字符串 |

除 `outputs` 外的所有输出节点接受以下公共属性：

| 属性 | 含义 |
| --- | --- |
| `name` | 具名节点的字段名，同级唯一 |
| `when` | 命名条件；为假时省略整个节点且不读取内容 |
| `secret` | `true` 时该节点的值在预览中脱敏 |
| `description` | 该节点/该行的预览说明，见第 11.4 节 |

顶层 `object` 还可声明导出元数据：

| 属性 | 含义 |
| --- | --- |
| `label` | 输出显示名称 |
| `filename` | 不含扩展名的安全文件名；扩展名由格式决定 |
| `command` | 只显示的启动命令；`{filename}` 替换为完整文件名 |

### 11.1. 标量

标量元素为 `string`、`boolean`、`integer`、`enum`。它们接受公共属性以及：

| 属性 | 适用 | 含义 |
| --- | --- | --- |
| `from` / `ref` | 全部 | 数据来源；与 `value`、子值元素三选一 |
| `value` | 全部 | 字面常量，按元素类型解码 |
| `transform` | 全部 | 输出前应用的转换序列 |
| `unit` | `string` | `s` 或 `ms`，为整数来源追加单位 |
| `options` | `enum` | 必需；引用顶层枚举输入 |

来源 MUST 恰好为 `from`、`ref`、字面 `value` 或恰好一个子值元素之一。处理顺序为
来源，然后 `transform`，然后 `unit`，最后类型校验。`enum` 的值 MUST 属于所引用的
顶层枚举；`unit` 只用于 `string`。

### 11.2. 对象与集合

`object` 接受公共属性以及零个或多个具名输出子元素，不改变输入作用域。

`list` 接受公共属性和可选 `from`、`where-field`、`equals`、`omit-empty`。有 `from`
时来源 MUST 为列表，各来源行依次成为当前行并保留顺序；没有 `from` 时在调用者行求值
模板一次，产生零项或一项。`where-field` 与 `equals` MUST 同时出现并要求 `from`，按
`eq` 的标量文本语义过滤。

`record` 接受公共属性、必需 `from` 和 `key`，以及可选 `key-transform`、`omit-empty`。
对来源列表的每行，先求值子元素，再读取 `key`、执行转换并要求非空字符串；转换后重复
的键 MUST 导致投影失败，绝不覆盖。动态键是普通字符串，不必是标识符。

`omit-empty` 仅用于 `list`、`record`，为真时省略已完成的空集合；默认假。对象和标量
不接受此属性。其他情况下，空对象和空集合保留为数据。假、零、空字符串 MUST NOT 隐含
省略；隐藏输出不读取来源。

### 11.3. 脱敏

预览脱敏作用于成功的投影：消费者 MUST 对照声明的输出结构检查值，拒绝未知字段或错误
类型，并把每个 `secret="true"` 节点替换为八个 U+2022 字符 `••••••••`。整个对象或
集合也可以标记为敏感；映射键仍可见，版本 5 没有敏感键标记。脱敏 MUST NOT 再次求值
条件、读取输入或修改投影，也不是导出或校验的输入。复制、下载始终使用原始输出。

### 11.4. 预览说明

`description` 是展示元数据，MUST NOT 参与投影、校验、脱敏或序列化，也 MUST NOT 被
解释为标记或代码。预览按输出结构逐行渲染，每行显示对应节点的 `description`：

- `object` 按成员名匹配；
- `list` 复用唯一的项模板；
- `record` 用动态键渲染每个条目并复用唯一子模板；
- `enum` 输出优先显示所选选项的 `description`，节点自身的 `description` 作为回退。

## 12. 路径与作用域

数据路径只能是 `/name`（根）或 `name`（当前行）。只接受单段名称，字符为 ASCII 字母、
数字、`_`、`-`；不支持点号路径、父级遍历、通配符或隐式数组索引。集合操作之外，行
等于根；输出对象不改变输入作用域，输出名称也不被解释为输入路径。

`when`、`use ref` 引用条件，其他 `ref` 属性引用命名值。命名条件继承调用者的行；
命名值始终把根和行都设为根求值，即使从集合内部调用。消费者 MUST 拒绝未声明符号和
引用不存在输入的路径，并 MUST 检测两个命名空间之间的循环依赖，不受定义是否使用
影响。

## 13. 限制与安全

描述 MUST NOT 超过 1,048,576 个 UTF-8 字节。`config-dsl` 元素深度从零开始，超过 64
的深度 MUST 被拒绝。各集合初始行数最多 1000。消费者 MUST 在递归/分配之前执行限制，
超限 MUST 明确失败，不得截断数据或部分导出。

描述 MUST NOT 发起文件访问、DNS、HTTP、子进程、环境变量展开或代码执行。输出路径仍是
供目标应用解释的数据。诊断 MUST NOT 包含提交的字段值或敏感源码片段，SHOULD 标明阶段
（描述、输入、投影、序列化）和安全的逻辑路径，并 SHOULD 使用从一开始的行列号。
生成配置不验证部署环境的 DNS、文件、防火墙、TLS 或连接。

## 14. 完整示例

此示例使用空敏感值，不是可部署的 TUIC 配置。

```xml
<config-dsl version="5" target-version="example">
  <ui title="示例配置" brand="Example" mark="E" format-field="encoding">
    <section name="general" label="常规"/>
  </ui>
  <inputs>
    <field name="encoding" type="enum" default="json" label="输出格式">
      <option value="json" label="JSON"/>
      <option value="toml" label="TOML"/>
    </field>
    <field name="host" type="string" default=" [2001:db8::1] " label="主机" section="general"/>
    <field name="port" type="string" default="0443" label="端口" section="general" widget="number"/>
    <field name="auth" type="boolean" default="false" label="认证" section="general"/>
    <field name="active" type="integer" default="0" label="选中行" section="general"/>
    <collection name="users" initial-items="1" section="general">
      <field name="key" type="string" default="demo" label="键"/>
      <field name="secret" type="string" default="" label="密钥" widget="password"/>
    </collection>
  </inputs>
  <conditions>
    <condition name="auth"><truthy from="/auth"/></condition>
  </conditions>
  <values>
    <value name="host"><source from="/host" transform="trim unbracket"/></value>
  </values>
  <outputs>
    <object name="example" label="示例" filename="example" command="example --config {filename}">
      <string name="server" description="服务端端点。">
        <endpoint><source ref="host"/><source from="/port" transform="integer"/></endpoint>
      </string>
      <boolean name="enabled" from="/auth" description="是否启用。"/>
      <list name="alpn" description="ALPN 列表。"><string value="h3" description="HTTP/3 标识。"/></list>
      <string name="selected" description="当前选中的用户键。">
        <select from="/users" index="/active"><source from="key"/></select>
      </string>
      <record name="users" from="/users" key="key" key-transform="trim lowercase" when="auth" description="用户映射。">
        <string from="secret" secret="true" description="用户密钥。"/>
      </record>
    </object>
  </outputs>
</config-dsl>
```

默认投影（`auth` 为假）：

```json
{"example":{"server":"[2001:db8::1]:443","enabled":false,"alpn":["h3"],"selected":"demo"}}
```

## 15. 参考资料

规范性引用为用于要求术语的 [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) 和
[RFC 8174](https://www.rfc-editor.org/rfc/rfc8174)，以及第 2.1 节使用的
[XML 1.0 字符范围](https://www.w3.org/TR/xml/#charsets)。其他词法行为由本文定义。

## 附录 A. 元素与属性速查（说明性）

顶层区块：`ui`、`validators`、`inputs`*、`conditions`、`values`、`outputs`*、`rules`、
`effects`（`*` 为必需）。

| 元素 | 关键属性 | 子元素 |
| --- | --- | --- |
| `config-dsl` | `version`、`target-version` | 顶层区块 |
| `ui` | `title`、`brand`、`mode-field`、`format-field` | `section`、`notice` |
| `section` | `name`、`label`、`detail`、`collapsed`、`when` | `notice` |
| `notice` | `text`、`when` | — |
| `field` | `name`、`type`、`default`、`label`、`widget`、`when`、`rule`、`generator` | `option` |
| `option` | `value`、`label`、`description` | — |
| `collection` | `name`、`initial-items`、`min-items`、`selected-by`、`all-when`、`select-when` | `field` |
| `condition` | `name` | 条件元素 |
| `value` | `name` | 值元素 |
| `validator` | `name`、`kind`、`message`、`min`、`max`、`transform`、`nonblank` | — |
| `assert` / `unique` | `key`、`message`、`when`、`collection` | 条件 / 值元素 |
| `reset` | `on`、`target` | — |
| `outputs` | — | 输出元素 |
| `object` | `name`、`when`、`secret`、`label`、`filename`、`command`、`description` | 输出元素 |
| `list` | `name`、`when`、`secret`、`from`、`where-field`、`equals`、`omit-empty`、`description` | 一个未命名输出元素 |
| `record` | `name`、`when`、`secret`、`from`、`key`、`key-transform`、`omit-empty`、`description` | 一个未命名输出元素 |
| `string` / `boolean` / `integer` / `enum` | `name`、`when`、`secret`、`from`/`ref`/`value`、`transform`、`unit`、`options`、`description` | 至多一个值元素 |
