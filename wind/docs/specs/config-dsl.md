Draft: config-dsl-02
Category: Experimental / Specification Draft
Date: September 2026
Language version: 5

# Config DSL: Static Configuration Description

[简体中文](../config-dsl.md)

## Status of This Memo

This is a proposed repository-level standard, not an adopted Wind API or an
Internet standard. It describes the static XML dialect used by the TUIC config
editor as a reference for other configuration editors. Wind does not yet
implement this DSL. The draft revision `config-dsl-02` and the language attribute
`version="5"` identify different things.

This document focuses on the XML document structure and field usage. Sections 1–13
are normative and Section 15 lists normative references; Section 14 and Appendix A
are informative. The English and Chinese editions have matching section numbers and
are maintained together.

## Abstract

Config DSL statically describes inputs, defaults, applicability conditions, output
structure, fixed conversions, and secret redaction. Consumers derive forms,
validate input, and project structured configuration values; TOML, JSON, and YAML
are downstream encodings. The description contains no executable code.

## 1. Scope and terminology

Uppercase **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** express the
requirement levels of BCP 14 ([RFC 2119](https://www.rfc-editor.org/rfc/rfc2119),
[RFC 8174](https://www.rfc-editor.org/rfc/rfc8174)). Lowercase uses are ordinary
prose.

| Term | Meaning |
| --- | --- |
| Description | One DSL document, independent of an input session |
| Consumer | An implementation that parses, validates, and projects a description |
| Host | Application providing editing, target-specific validation, and export |
| Root | Object containing top-level input fields and collections |
| Row | Current input object; initially the root, replaced during collection mapping or selection |
| Projection | Structured output produced from one input snapshot |

The DSL does not replace a target application's configuration parser or wire
protocol. Imports, migrations, includes, arbitrary functions, assignment,
recursion, network lookups, and executable expressions are outside version 5.
Consumers MUST NOT evaluate strings as Rust, JavaScript, Shell, template, or any
other programming language. Serde, quick-xml, and Rust are implementation choices,
not language requirements.

## 2. Lexical format

### 2.1. Encoding and structure syntax

Documents MUST be UTF-8 without a BOM. Allowed characters are U+0009, U+000A,
U+000D, U+0020–U+D7FF, U+E000–U+FFFD, and U+10000–U+10FFFF, the `Char` range of
[XML 1.0](https://www.w3.org/TR/xml/#charsets). This language is an XML variant,
not a general XML processor, and introduces no other XML processing rules.

Names are case-sensitive; start and end tags and attribute quotes MUST match.
Duplicate attributes, non-whitespace text between elements, and extra content after
the root element MUST be rejected. Comments are allowed between elements, but a
comment MUST NOT contain `--`. XML declarations, DTDs, entity declarations,
namespaces, processing instructions, CDATA, and mixed content MUST be rejected.

The following PEG uses `~` for sequencing, `|` for ordered choice, `*`/`+` for
repetition, `!` for negative lookahead, and `ANY` for one allowed character. Tag
matching, entity validity, and semantic constraints are checked outside this
grammar.

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

### 2.2. Attribute decoding and literal syntax

The only named entities are `&amp;`, `&lt;`, `&gt;`, `&quot;`, and `&apos;`.
Decimal `&#DIGITS;` and hexadecimal `&#xHEXDIGITS;` references MUST have at least
one digit and point to an allowed character. Symbols, unknown entities, missing
semicolons, surrogate code points, and out-of-range code points MUST be rejected.
Decoding happens once: `&amp;lt;` yields the text `&lt;`, not `<`.

Decoded whitespace, including literal tabs and newline sequences, MUST be
preserved. Consumers MUST NOT implicitly perform XML attribute whitespace
normalization, newline rewriting, Unicode normalization, or trimming. General XML
libraries may need adaptation.

| Decoded form | Syntax |
| --- | --- |
| Identifier | `[A-Za-z_][A-Za-z0-9_-]*` |
| Boolean | only `true` or `false` |
| Unsigned integer | `[0-9]+`, decimal |
| Signed integer | `-?[0-9]+`, decimal |

Leading zeros are allowed; a plus sign, fraction, exponent, or surrounding
whitespace is not. The value `-0` equals zero. Tags, plain strings, and enum option
values need not be identifiers.

## 3. Document structure

The root MUST be `config-dsl` with only the required `version="5"` and
`target-version` attributes. `target-version` is opaque metadata, not a language
selector.

The root contains the following top-level blocks. Unknown or duplicate blocks MUST
be rejected; block order is irrelevant, all blocks except `inputs` and `outputs`
may be omitted, and each block may appear at most once. Except for `ui`, these
blocks MUST NOT have attributes.

| Block | Required | Purpose |
| --- | --- | --- |
| `ui` | no | Interface metadata: branding, sections, notices |
| `validators` | no | Reusable field validators |
| `inputs` | yes | Top-level input fields and collections |
| `conditions` | no | Named conditions |
| `values` | no | Named values |
| `outputs` | yes | Output structure and export metadata |
| `rules` | no | Cross-field constraints |
| `effects` | no | Field-linkage resets |

## 4. Interface `ui`

`ui` describes page branding, sections, and notices. All attributes are optional,
but `title` and `brand` are required by the reference implementation.

| Attribute | Meaning |
| --- | --- |
| `title` | Page title |
| `brand` | Brand name |
| `mark` | Brand mark character |
| `eyebrow` | Small text above the title |
| `description` | Text below the title |
| `export-hint` | Hint for the export area |
| `mode-field` | References any top-level `enum` field, rendered as mode buttons |
| `format-field` | References any top-level `enum` field, rendered as the output-format selector |

The fields referenced by `mode-field` and `format-field` MUST be top-level `enum`
fields. The `format-field` options MUST be exactly `json`, `toml`, and `yaml`; when
no format is bound, JSON is the default.

`ui` children are `section` and `notice`.

`section` declares an interface section:

| Attribute | Required | Meaning |
| --- | --- | --- |
| `name` | yes | Section identifier, bound to an input's `section` |
| `label` | yes | Section title |
| `detail` | no | Supplementary title text |
| `collapsed` | no | `true` uses a collapsed panel; default `false` |
| `when` | no | Named condition controlling the whole section |

`notice` displays plain text:

| Attribute | Required | Meaning |
| --- | --- | --- |
| `text` | yes | Notice text, rendered as text and never parsed as HTML |
| `when` | no | Named condition controlling visibility |

A `notice` under `ui` appears in the export area; one inside a `section` appears in
that section. When no `section` is declared, consumers generate generic sections
from input `section` names; once sections are declared, every ordinary field and
collection MUST bind to a declared section.

## 5. Inputs `inputs`

`inputs` children are `field` and `collection`. Top-level fields and collections
share one namespace and their names MUST be unique.

### 5.1. Field `field`

| Attribute | Required | Meaning |
| --- | --- | --- |
| `name` | yes | Input identifier |
| `type` | yes | `string`, `boolean`, `integer`, or `enum` |
| `default` | yes | Initial value decoded by `type` |
| `label` | yes | Plain-text label |
| `placeholder` | no | Placeholder text |
| `hint` | no | Field explanation |
| `section` | no | Owning interface section |
| `widget` | no | `text`, `password`, `number`, or `email`; `string` only |
| `when` | no | Named condition controlling both visibility and validation |
| `rule` | no | Referenced validator name |
| `generator` | no | `uuid-v4` or `hex`; `string` only |
| `bytes` | no | `hex` only, random byte count 1–1024, default 24 |

| Type | Default value | Control |
| --- | --- | --- |
| `string` | The literal string | Text input; `widget` selects the specific control |
| `boolean` | only `true` / `false` | Checkbox |
| `integer` | Non-negative integer | Number input |
| `enum` | MUST belong to a child `option` | Select |

An `enum` MUST contain one or more `option` children; a non-enum field MUST NOT have
children.

`option`:

| Attribute | Required | Meaning |
| --- | --- | --- |
| `value` | yes | Enum value, unique within the field |
| `label` | yes | Option text |
| `description` | no | Preview explanation for this value, see Section 11.4 |

`widget` does not change the data type: `type="string" widget="number"` keeps
editable text. A password widget does not imply output redaction; redaction is
declared by an output's `secret`.

`generator` produces random values on first load, on row creation, and when the
generate button is triggered: `uuid-v4` uses exactly 16 bytes and sets the version
and variant bits; `hex` uses the byte count given by `bytes`. XML defaults never
store generated credentials.

A false `when` excludes the field from display and ordinary validation, but MUST NOT
clear its stored value. Applicability is not access control and does not forbid an
output from reading the field.

### 5.2. Collection `collection`

`collection` declares a group of addable and removable homogeneous rows; row fields
reuse the `field` structure of Section 5.1 (row-level layout attributes such as
`section` and `hint` are not rendered inside a row).

| Attribute | Required | Meaning |
| --- | --- | --- |
| `name` | yes | Collection identifier |
| `initial-items` | yes | Initial row count, 0–1000 |
| `label` | no | Row name, defaulting to `name` |
| `section` | no | Owning interface section |
| `hint` | no | Collection explanation |
| `add-label` | no | Add-button text, default “添加” |
| `generate-label` | no | Generate-button text, default “生成随机值” |
| `min-items` | no | Minimum rows kept, 0–1000, not exceeding `initial-items` |
| `selected-by` | no | References a top-level `integer` field, showing a row selector |
| `all-when` | no | When true, all rows are editable and validated; otherwise only the selected row |
| `select-when` | no | Condition enabling the row selector and index validation; enabled by default |
| `when` | no | Collection visibility and validation condition |

`selected-by` MUST reference a top-level `integer` field, and each field may be used
by at most one collection; `all-when` and `select-when` are only available when
`selected-by` is declared. A collection `when` is evaluated at the root; a field
`when` is evaluated in each applicable row. Removing the selected row resets the
index to zero; removing an earlier row decrements it. Stable UI row identity,
selection controls, credential generation, and removal are host behavior and are
not exported automatically.

## 6. Conditions `conditions`

`conditions` contains `condition name="ID"`, each with exactly one condition child.
Condition names MUST be unique. Condition elements cannot have `name`, `when`, or
conversions:

| Element | Attributes | Children and result |
| --- | --- | --- |
| `all` / `any` | none | One or more conditions; all/any are true |
| `not` | none | Exactly one condition; negated |
| `use` | required `ref` | No children; references a named condition |
| `eq` | exactly one of `from`/`ref`, required `value` | No children; compares scalar text to a literal |
| `truthy` | exactly one of `from`/`ref` | No children; requires and returns a boolean |
| `ip` | exactly one of `from`/`ref` | No children; checks an IPv4/IPv6 literal |
| `valid` | exactly one of `from`/`ref`, required `rule` | No children; applies a declared validator |
| `compare` | required `op="ne|gte"` | Exactly two value elements; non-negative integer comparison or inequality |

`all`/`any` MUST short-circuit left to right and propagate errors instead of
treating them as false. `eq` compares scalar text: strings are preserved, booleans
render as `true`/`false`, and integers use canonical decimal; the string `"01"` does
not equal `value="1"`, while the integer 1 does. `truthy` does not convert numbers
or strings. `ip` does not trim, unbracket, or query DNS.

The `when` attribute of any node (field, collection, output, rule, and so on) MUST
reference a declared condition; undeclared references MUST be rejected.

## 7. Values `values`

`values` contains `value name="ID"`, each with exactly one value child. Value names
MUST be unique and live in a namespace separate from condition names. Every value
element accepts an optional `when`, checked before any source or child is
evaluated; when false it produces a missing value.

| Element | Attributes beyond `when` | Children |
| --- | --- | --- |
| `source` | exactly one of `from`/`ref`, optional `transform` | none |
| `coalesce` | none | one or more value elements |
| `endpoint` | none | exactly two value elements: host and port |
| `select` | required `from`, `index` | exactly one value element |

`coalesce` returns the first result that is neither missing nor an empty string,
preserving false, zero, empty lists, and empty objects; with no candidate it returns
an empty string. `endpoint` requires a host string and an integer port 1–65535 and
produces `[host]:port` for an unbracketed IPv6 host, otherwise `host:port`. `select`
reads a list and a zero-based non-negative integer index in the caller's context,
then evaluates its child with the selected row; missing data, type errors, or an
out-of-range index MUST be rejected.

### 7.1. Fixed conversions `transform`

When present, `transform` and `key-transform` contain a non-empty ordered sequence
of operations separated by whitespace; absent means no conversion. Each operation
requires a string input:

| Operation | Result |
| --- | --- |
| `trim` | Removes leading and trailing whitespace |
| `lowercase` | Locale-independent lowercase conversion |
| `unbracket` | Removes one pair of leading `[` and trailing `]`, else keeps the text |
| `integer` | Parses a decimal non-negative integer |
| `socket` | Normalizes an IP endpoint into a comparison key |
| `port` | Extracts the integer port from an IP endpoint |

There is no implicit type conversion; a string operation after `integer` fails
because the intermediate value is a number. `unit` applies only to `string` outputs
and MUST be `s` or `ms`: the converted value must be an integer, and the suffix is
appended to its canonical decimal form without scaling.

## 8. Validators `validators`

`validators` contains `validator`, defining reusable field checks:

| Attribute | Required | Meaning |
| --- | --- | --- |
| `name` | yes | Validator identifier, unique |
| `kind` | yes | Check type, see the table |
| `message` | yes | Error text on failure |
| `min` / `max` | no | Non-negative integer range |
| `transform` | no | Conversion applied before checking |
| `nonblank` | no | When `true`, requires a string that is non-empty after trimming |

| kind | Check |
| --- | --- |
| `required` | String is non-empty after trimming |
| `length` | UTF-8 byte count of a string or item count of a list satisfies min/max |
| `integer` | Parses as a safe non-negative decimal u64 and applies min/max |
| `optional-integer` | Empty string passes; otherwise checked as `integer` |
| `host` | Connectable domain or IP, not a wildcard listen address |
| `socket` | IP:port |
| `endpoint` | Domain or IP:port |
| `email` | Basic email format |
| `uuid` | Non-nil, standard hyphenated UUID |
| `domain` | DNS name |
| `public-domain` | DNS name containing a dot |
| `loopback-socket` | The listen endpoint's IP is a loopback address |

A field `rule` MUST reference a declared validator; an undeclared rule MUST be
rejected, never silently skipped. These checks are syntactic and do not query DNS or
prove reachability, deliverability, or TLS trust.

## 9. Cross-field rules `rules`

`rules` contains `assert` or `unique`:

| Attribute | Required | Meaning |
| --- | --- | --- |
| `key` | yes | Field the error binds to |
| `message` | yes | Failure text |
| `when` | no | Named condition |
| `collection` | no | Collection checked row by row |

- `assert` has exactly one condition child; a false condition reports an error.
- `unique` has one or more value children; for visible rows that participate in
  validation, the key formed by these values MUST be unique.

`key` is the error-binding field: a top-level error uses the field name, while a row
error automatically forms `collection.index.field`. An existing error on the same
field is preserved. A `valid` condition can exclude numbers or endpoints that are
not yet legal before comparison.

## 10. Effects `effects`

`effects` contains `reset`:

| Attribute | Required | Meaning |
| --- | --- | --- |
| `on` | yes | Trigger field |
| `target` | yes | Field to reset |

When the actual value of the `on` field changes, `target` is restored to its XML
default. Scripts are not supported, and resets are not triggered recursively. `on`
and `target` MUST reference declared top-level fields.

## 11. Outputs `outputs`

`outputs` has no attributes; its children are top-level outputs, usually named
`object`s that each correspond to an exported file.

| Output element | Purpose |
| --- | --- |
| `object` | Object; child names are unique |
| `list` | List; exactly one unnamed child as the item template |
| `record` | Dynamic-key map; exactly one unnamed child |
| `string` / `boolean` / `integer` | Strict scalar types |
| `enum` | String drawn from a top-level enum |

Every output node except `outputs` accepts these common attributes:

| Attribute | Meaning |
| --- | --- |
| `name` | Field name of a named node, unique among siblings |
| `when` | Named condition; when false the whole node is omitted without reading its contents |
| `secret` | When `true`, the node's value is redacted in the preview |
| `description` | Preview explanation for this node or row, see Section 11.4 |

A top-level `object` may also declare export metadata:

| Attribute | Meaning |
| --- | --- |
| `label` | Output display name |
| `filename` | Safe file name without extension; the extension follows the format |
| `command` | Display-only launch command; `{filename}` is replaced by the full file name |

### 11.1. Scalars

Scalar elements are `string`, `boolean`, `integer`, and `enum`. They accept the
common attributes plus:

| Attribute | Applies to | Meaning |
| --- | --- | --- |
| `from` / `ref` | all | Data source; exactly one of this, `value`, or a child value element |
| `value` | all | Literal constant decoded by element type |
| `transform` | all | Conversion sequence applied before output |
| `unit` | `string` | `s` or `ms`, appending a unit to an integer source |
| `options` | `enum` | Required; references a top-level enum input |

The source MUST be exactly one of `from`, `ref`, a literal `value`, or exactly one
child value element. The processing order is source, then `transform`, then `unit`,
then type validation. An `enum` value MUST belong to the referenced top-level enum,
and `unit` applies only to `string`.

### 11.2. Objects and collections

`object` accepts the common attributes plus zero or more named output children and
does not change the input scope.

`list` accepts the common attributes and optional `from`, `where-field`, `equals`,
and `omit-empty`. With `from`, the source MUST be a list and each source row becomes
the current row in order; without `from`, the template is evaluated once in the
caller's row, producing zero or one item. `where-field` and `equals` MUST appear
together and require `from`, filtering by the scalar-text semantics of `eq`.

`record` accepts the common attributes, required `from` and `key`, and optional
`key-transform` and `omit-empty`. For each source-list row, the child is evaluated
first, then `key` is read, transformed, and required to be a non-empty string; a key
that repeats after normalization MUST fail projection and never overwrite. Dynamic
keys are plain strings and need not be identifiers.

`omit-empty` applies only to `list` and `record` and, when true, omits a completed
empty collection; the default is false. Objects and scalars do not accept it.
Otherwise, empty objects and collections are preserved as data. False, zero, and
empty strings MUST NOT imply omission; a hidden output does not read its source.

### 11.3. Redaction

Preview redaction applies to a successful projection: consumers MUST check values
against the declared output structure, reject unknown fields or wrong types, and
replace every `secret="true"` node with eight U+2022 characters, `••••••••`. Entire
objects or collections may be marked secret; record keys remain visible, and version
5 has no secret-key marker. Redaction MUST NOT re-evaluate conditions, read inputs,
or modify the projection, and is not an input to export or validation. Copy and
download always use the original output.

### 11.4. Preview descriptions

`description` is presentation metadata and MUST NOT participate in projection,
validation, redaction, or serialization, and MUST NOT be interpreted as markup or
code. The preview renders one line per output node, showing that node's
`description`:

- `object` matches by member name;
- `list` reuses its single item template;
- `record` renders each entry by dynamic key and reuses its single child template;
- an `enum` output prefers the selected option's `description`, falling back to the
  node's own `description`.

## 12. Paths and scopes

A data path is either `/name` (root) or `name` (current row). Only a single name
segment is accepted, using ASCII letters, digits, `_`, and `-`; dot paths, parent
traversal, wildcards, and implicit array indexes are not supported. Outside
collection operations, the row equals the root; an output object does not change the
input scope, and output names are not interpreted as input paths.

`when` and `use ref` reference conditions; other `ref` attributes reference named
values. A named condition inherits the caller's row; a named value always evaluates
with both root and row set to the root, even when called from inside a collection.
Consumers MUST reject undeclared symbols and paths that reference no declared input,
and MUST detect cyclic dependencies between the two namespaces regardless of whether
a definition is used.

## 13. Limits and safety

A description MUST NOT exceed 1,048,576 UTF-8 bytes. The `config-dsl` element depth
starts at zero, and a depth greater than 64 MUST be rejected. Collections have at
most 1000 initial rows. Consumers MUST enforce limits before recursion or
allocation; exceeding a limit MUST fail explicitly without truncating data or
partially exporting.

A description MUST NOT perform file access, DNS, HTTP, subprocesses, environment
expansion, or code execution. Output paths remain data for the target application to
interpret. Diagnostics MUST NOT include submitted field values or sensitive source
fragments, SHOULD name the stage (description, input, projection, serialization) and
a safe logical path, and SHOULD use one-based line and column numbers. A generated
configuration does not verify the deployment environment's DNS, files, firewall,
TLS, or connectivity.

## 14. Example

This example uses empty secret data and is not a deployable TUIC configuration.

```xml
<config-dsl version="5" target-version="example">
  <ui title="Example" brand="Example" mark="E" format-field="encoding">
    <section name="general" label="General"/>
  </ui>
  <inputs>
    <field name="encoding" type="enum" default="json" label="Output format">
      <option value="json" label="JSON"/>
      <option value="toml" label="TOML"/>
    </field>
    <field name="host" type="string" default=" [2001:db8::1] " label="Host" section="general"/>
    <field name="port" type="string" default="0443" label="Port" section="general" widget="number"/>
    <field name="auth" type="boolean" default="false" label="Authentication" section="general"/>
    <field name="active" type="integer" default="0" label="Selected row" section="general"/>
    <collection name="users" initial-items="1" section="general">
      <field name="key" type="string" default="demo" label="Key"/>
      <field name="secret" type="string" default="" label="Secret" widget="password"/>
    </collection>
  </inputs>
  <conditions>
    <condition name="auth"><truthy from="/auth"/></condition>
  </conditions>
  <values>
    <value name="host"><source from="/host" transform="trim unbracket"/></value>
  </values>
  <outputs>
    <object name="example" label="Example" filename="example" command="example --config {filename}">
      <string name="server" description="Server endpoint.">
        <endpoint><source ref="host"/><source from="/port" transform="integer"/></endpoint>
      </string>
      <boolean name="enabled" from="/auth" description="Whether the feature is enabled."/>
      <list name="alpn" description="ALPN list."><string value="h3" description="HTTP/3 identifier."/></list>
      <string name="selected" description="Selected user key.">
        <select from="/users" index="/active"><source from="key"/></select>
      </string>
      <record name="users" from="/users" key="key" key-transform="trim lowercase" when="auth" description="User map.">
        <string from="secret" secret="true" description="User secret."/>
      </record>
    </object>
  </outputs>
</config-dsl>
```

Default projection (`auth` false):

```json
{"example":{"server":"[2001:db8::1]:443","enabled":false,"alpn":["h3"],"selected":"demo"}}
```

## 15. References

Normative references are [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) and
[RFC 8174](https://www.rfc-editor.org/rfc/rfc8174) for requirement terminology, and
[XML 1.0 character ranges](https://www.w3.org/TR/xml/#charsets) for Section 2.1.
All other lexical behavior is defined here.

## Appendix A. Element and attribute quick reference (informative)

Top-level blocks: `ui`, `validators`, `inputs`*, `conditions`, `values`, `outputs`*,
`rules`, `effects` (`*` required).

| Element | Key attributes | Children |
| --- | --- | --- |
| `config-dsl` | `version`, `target-version` | top-level blocks |
| `ui` | `title`, `brand`, `mode-field`, `format-field` | `section`, `notice` |
| `section` | `name`, `label`, `detail`, `collapsed`, `when` | `notice` |
| `notice` | `text`, `when` | — |
| `field` | `name`, `type`, `default`, `label`, `widget`, `when`, `rule`, `generator` | `option` |
| `option` | `value`, `label`, `description` | — |
| `collection` | `name`, `initial-items`, `min-items`, `selected-by`, `all-when`, `select-when` | `field` |
| `condition` | `name` | condition element |
| `value` | `name` | value element |
| `validator` | `name`, `kind`, `message`, `min`, `max`, `transform`, `nonblank` | — |
| `assert` / `unique` | `key`, `message`, `when`, `collection` | condition / value elements |
| `reset` | `on`, `target` | — |
| `outputs` | — | output elements |
| `object` | `name`, `when`, `secret`, `label`, `filename`, `command`, `description` | output elements |
| `list` | `name`, `when`, `secret`, `from`, `where-field`, `equals`, `omit-empty`, `description` | one unnamed output element |
| `record` | `name`, `when`, `secret`, `from`, `key`, `key-transform`, `omit-empty`, `description` | one unnamed output element |
| `string` / `boolean` / `integer` / `enum` | `name`, `when`, `secret`, `from`/`ref`/`value`, `transform`, `unit`, `options`, `description` | at most one value element |
