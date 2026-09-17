# 服务端配置

`tuic-server` 读取单个配置文件，字段以 TOML、JSON5 或 YAML 表示。本页列出 2.0.0-dev7 的全部字段、默认值与行为；**未生效字段**单独标注，不应作为可用功能依赖。

## 配置文件与加载 {#format}

- 配置文件格式由扩展名推断：`.toml`、`.json`、`.json5`、`.yaml`、`.yml`。
- 环境变量 `TUIC_FORCE_TOML` 强制按 TOML 解析；`TUIC_CONFIG_FORMAT` 显式指定格式（`toml`、`json`、`json5`、`yaml`、`yml`）。
- 容器中设置 `IN_DOCKER=true` 时，扩展名不可用会按内容推断格式。
- 命令行：

  | 参数 | 含义 |
  | --- | --- |
  | `-c`、`--config <PATH>` | 指定配置文件。 |
  | `-d`、`--dir <DIR>` | 指定目录，使用其中按字母序第一个可识别的配置文件。 |
  | `-i`、`--init` | 在当前目录生成示例 `config.toml`；文件已存在时拒绝覆盖。 |

`--config` 优先于 `--dir`；两者都未提供时报错退出。

配置采用 `deny_unknown_fields`：拼写错误或未知键会导致启动失败，而不是被忽略。

## 顶层字段 {#top-level}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `log_level` | 枚举 | `info` | `trace`、`debug`、`info`、`warn`、`error`、`off`。 |
| `log` | 表 | 见 [`[log]`](#log) | 日志输出设置。 |
| `server` | 套接字 | `[::]:8443` | QUIC/UDP 监听地址。 |
| `users` | 映射 | `{}` | UUID → 密码。至少需要一个用户。 |
| `tls` | 表 | 见 [`[tls]`](#tls) | TLS 与证书。 |
| `data_dir` | 路径 | `""`（当前工作目录） | 相对证书/密钥路径与 ACME 状态的基准目录，不存在时会创建。 |
| `backend` | 表 | 见 [`[backend]`](#backend) | QUIC 后端与传输参数。 |
| `zero_rtt_handshake` | 布尔 | `false` | 允许 QUIC 0-RTT 早期数据。可能被重放，默认关闭。 |
| `auth_timeout` | 时长 | `3s` | 等待客户端认证命令的时间。 |
| `stream_timeout` | 时长 | `60s` | TCP/UDP I/O 任务的保留时间。 |
| `outbound` | 表 | 见 [`[outbound]`](#outbound) | 出站规则。 |
| `acl` | 规则 | `[]` | 旧式 ACL 规则，见 [`[acl]` 与 `rules`](#acl)。 |
| `rules` | 规则 | `[]` | Metacubex 风格路由规则，在 ACL 之后求值。 |
| `experimental` | 表 | 见 [`[experimental]`](#experimental) | 安全防护开关。 |
| `dns` | 表 | 见 [`[dns]`](#dns) | 出站 DNS 解析。 |
| `geodata` | 表 | 见 [`[geodata]`](#geodata) | GEOIP/GEOSITE 数据库。 |
| `restful` | 表 | 见 [`[restful]`](#restful) | 管理 API。 |
| `masquerade` | 表 | 见 [`[masquerade]`](#masquerade) | HTTP/3 伪装反向代理。 |

时长字段使用 `humantime` 形式，如 `"3s"`、`"500ms"`、`"1m"`。

## `[log]` {#log}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `format` | 枚举 | `text` | `text` 或 `json`。 |
| `compact` | 布尔 | `true` | 单行紧凑输出，仅对 `text` 生效。 |
| `log_file` | 路径 | 无 | 可选文件输出，与 stdout 同时写入。 |
| `log_rotation` | 枚举 | `never` | 文件轮转：`never`、`hourly`、`daily`。 |

!!! warning "部分 `[log]` 会解析失败"
    `[log]` 未实现字段级默认。如果提供该表，请写全四个字段；省略整个 `[log]` 表时使用上表默认值。

## `[tls]` {#tls}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `self_sign` | 布尔 | `false` | 启动时按 `hostname` 生成自签名证书。 |
| `certificate` | 路径 | `""` | 证书链文件；相对路径基于 `data_dir`。 |
| `private_key` | 路径 | `""` | 私钥文件；相对路径基于 `data_dir`。 |
| `alpn` | 字符串列表 | `[]` | ALPN 列表。 |
| `hostname` | 字符串 | `localhost` | 证书 CN/SAN、SNI 与 ACME 域名。 |
| `auto_ssl` | 布尔 | `false` | 启用内置 ACME 自动申请与续期。 |
| `acme_email` | 字符串 | `""` | ACME 联系邮箱。 |
| `acme_staging` | 布尔 | `false` | 使用 ACME 预发布环境（测试用）。 |

三种证书方式：

- **已有证书**：设置 `certificate` 与 `private_key`。
- **ACME**：`auto_ssl = true`，并设置公开域名 `hostname`。需要通过 **TCP 80** 完成 HTTP-01 验证；`data_dir` 必须可写并持久化。以非 root 运行且需要绑定 80 端口时，可对二进制设置 `setcap CAP_NET_BIND_SERVICE=+eip`。
- **自签名**：`self_sign = true`，仅用于受控测试；客户端必须显式跳过校验。

!!! warning "ALPN 必须两端一致"
    服务端原样使用 `tls.alpn`，为空时不声明任何 ALPN；客户端为空时回退到 `h3`。建议两端都显式设置 `alpn = ["h3"]`。

证书在启动时加载一次。源码中存在证书解析器组件，但当前二进制未接入热重载，替换证书需要重启。

## `[backend]` {#backend}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `mode` | 枚举 | `quinn` | `quinn` 或 `quiche`。`quiche` 需要构建时启用 `quiche` cargo feature，否则启动失败。 |

### `[backend.quinn]`（默认后端）

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `congestion_control.controller` | 枚举 | `bbr` | `bbr`、`bbr3`、`cubic`、`newreno`。 |
| `congestion_control.initial_window` | 整数 | `1048576` | 初始拥塞窗口（字节）。 |
| `initial_mtu` | 整数 | `1200` | 初始 UDP 载荷大小。 |
| `min_mtu` | 整数 | `1200` | 保证支持的最小 MTU。 |
| `gso` | 布尔 | `true` | 启用 UDP Generic Segmentation Offload。 |
| `pmtu` | 布尔 | `true` | 启用路径 MTU 发现。 |
| `send_window` | 整数 | `16777216` | 未经确认可发送的最大字节数。 |
| `receive_window` | 整数 | `8388608` | 每条流可接收的最大字节数。 |
| `max_idle_time` | 时长 | `30s` | 空闲连接关闭等待时间。 |

!!! note "bbr 与 bbr3"
    当前实现中 `bbr` 与 `bbr3` 共用同一个 BBR 工厂，行为相同，不能据此承诺性能差异。序列化值为 `newreno`。

### `[backend.quiche]`（实验性）

| 字段 | 类型 | 默认值 |
| --- | --- | --- |
| `congestion_control.controller` | 枚举 | `bbr` |
| `congestion_control.initial_window` | 整数 | `1048576`（quiche 路径未使用） |
| `max_idle_time` | 时长 | `30s` |
| `max_concurrent_bi_streams` | 整数 | `100` |
| `max_concurrent_uni_streams` | 整数 | `100` |
| `send_window` | 整数 | `16777216` |
| `receive_window` | 整数 | `8388608` |
| `zero_rtt` | 布尔 | `false` |

## `[outbound]` {#outbound}

`[outbound.default]` 是未指定名称时使用的出站；`[outbound.<name>]` 为具名出站，可被 ACL 或规则引用。

| 字段 | 类型 | 默认值 | 适用 | 说明 |
| --- | --- | --- | --- | --- |
| `type` | 字符串 | `direct` | 全部 | `direct` 或 `socks5`；未知值回退到 `direct` 并告警。 |
| `ip_mode` | 枚举 | `v4first` | direct | `v4first`、`v6first`、`v4only`、`v6only`；兼容旧别名 `prefer_v4`、`only_v4` 等。 |
| `bind_ipv4` | 地址或列表 | 无 | direct | 源 IPv4 绑定；多个地址会创建轮询负载均衡出站。 |
| `bind_ipv6` | 地址或列表 | 无 | direct | 源 IPv6 绑定，行为同上。 |
| `bind_device` | 字符串 | 无 | direct | 绑定的网络接口（Linux）。 |
| `tfo` | 布尔 | 无 | direct | 启用 TCP Fast Open。 |
| `routing_mark` | 整数 | 无 | direct | Linux `SO_MARK`，用于策略路由。 |
| `addr` | 字符串 | 无 | socks5 | SOCKS5 服务器地址。 |
| `username` / `password` | 字符串 | 无 | socks5 | SOCKS5 凭据。 |
| `allow_udp` | 布尔 | 无（阻止） | socks5 | 允许 UDP；UDP 仍直连，不经 SOCKS5。 |

```toml
[outbound.default]
type = "direct"
ip_mode = "v4first"

[outbound.through_socks5]
type = "socks5"
addr = "127.0.0.1:1080"
username = "user"
password = "pass"
```

## `[acl]` 与 `rules` {#acl}

ACL 使用旧式规则，在 Metacubex `rules` 之前求值；两者都可用。无匹配时流量交给 `outbound.default`。

### `[acl]` 规则语法

每条规则为 `出站 [地址] [端口] [劫持地址]`，可写成 `[[acl]]` 表数组，或一个多行字符串：

```toml
acl = """
allow localhost udp/53
allow localhost udp/53,tcp/80,tcp/443
reject 10.6.0.0/16
allow *.google.com
reject localhost
custom_outbound_name example.com 80,443
default 8.8.4.4 udp/53 1.1.1.1
"""
```

或：

```toml
[[acl]]
addr = "127.0.0.1"
ports = "udp/53"
outbound = "default"
hijack = "1.1.1.1"
```

- 地址：单个 IP、CIDR、域名、通配域名（`*.example.com` 或 `suffix:`）、`localhost`、`private`、`*`。`localhost` 展开为 `127.0.0.0/8` 与 `::1/128`；`private` 展开为 RFC1918、链路本地、回环与 ULA 网段。
- 端口：`443`、`80-90`，可加 `tcp/` 或 `udp/` 前缀；逗号分隔，未写协议时同时匹配 TCP 与 UDP。
- 出站：`allow`/`default` 指向 `default`；`reject`/`block`/`deny` 拒绝；其他名称引用具名出站。

!!! warning "`hijack` 尚未生效"
    旧式 ACL 会解析并保存 `hijack`，但转换器当前不读取该字段，重定向不会执行。

### `rules`

Metacubex 风格规则字符串，在 ACL 之后求值：

```toml
rules = [
  "DOMAIN-SUFFIX,google.com,proxy",
  "IP-CIDR,10.0.0.0/8,direct,no-resolve",
  "DST-PORT,443,proxy",
  "MATCH,proxy",
]
```

支持 `DOMAIN`、`DOMAIN-SUFFIX`、`DOMAIN-KEYWORD`、`IP-CIDR`、`IP-CIDR6`、`DST-PORT`、`NETWORK`、`GEOIP`、`GEOSITE`、`MATCH` 等；`IP-ASN` 当前不支持。目标为 `reject`/`block`/`deny` 时拒绝，否则作为出站名称。`GEOIP`/`GEOSITE` 依赖 [`[geodata]`](#geodata)，缺少数据库时规则永不匹配并记录告警。

## `[dns]` {#dns}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `mode` | 枚举 | `system` | `system`、`cloudflare`、`cloudflare-tls`、`cloudflare-https`、`google`、`google-tls`、`google-https`、`quad9`、`quad9-tls`、`quad9-https`、`custom`。 |
| `servers` | 字符串列表 | `[]` | `mode = "custom"` 时使用。 |
| `timeout` | 时长 | 库默认 | 单次查询超时。 |
| `attempts` | 整数 | 库默认 | 查询重试次数。 |
| `stack_prefer` | 枚举 | `v4first` | A/AAAA 排序偏好。 |

`servers` 支持 `1.1.1.1`、`udp://`、`tcp://`、`tls://`（DoT，默认 853）、`https://`（DoH，默认 443），IPv6 加端口时使用方括号：

```toml
[dns]
mode = "custom"
servers = ["tls://1.1.1.1#cloudflare-dns.com", "https://[2606:4700:4700::1111]:443#cloudflare-dns.com"]
```

默认 `system` 使用操作系统解析器，保持向后兼容。

## `[geodata]` {#geodata}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `geosite` | 路径 | 无 | v2ray `geosite.dat`。 |
| `geoip` | 路径 | 无 | v2ray `geoip.dat`。 |

两者都设置时才构建缓存（位于 `data_dir/geodata.cache`）。

## `[restful]` {#restful}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `enabled` | 布尔 | `false` | 启动管理 API。 |
| `addr` | 套接字 | `127.0.0.1:13471` | 监听地址。 |
| `secret` | 字符串 | `""` | Bearer 令牌；为空表示不鉴权。 |
| `maximum_clients_per_user` | 整数 | `0` | 每用户最大并发（0 表示不限）。 |

端点（`Authorization: Bearer <secret>`）：

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| POST | `/kick` | 请求体为 UUID 数组，返回 `{"kicked": n}`。 |
| GET | `/online` | 在线用户及连接数。 |
| GET | `/detailed_online` | 在线用户及远端地址。 |
| GET | `/traffic` | 每用户累计 `{tx, rx, requests}`。 |
| GET | `/reset_traffic` | 重置并返回每用户增量。 |

!!! warning "`maximum_clients_per_user` 当前未强制"
    该字段会构造连接计数注册表，但当前没有拒绝超限连接的钩子，不能依赖它限制并发。

## `[masquerade]` {#masquerade}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `enabled` | 布尔 | `false` | 启用 HTTP/3 伪装。 |
| `upstream` | 字符串 | `https://example.com` | 反向代理目标站点。 |

启用后，首条流的首字节不是 TUIC 版本 `0x05` 的连接（例如主动探测的 HTTP/3 客户端）会被反向代理到 `upstream`，使服务器表现为普通 HTTP/3 网站。

## `[experimental]` {#experimental}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `drop_loopback` | 布尔 | `true` | 解析目标后拒绝回环地址。 |
| `drop_private` | 布尔 | `true` | 拒绝私有、链路本地与 ULA 地址。 |

!!! danger "默认拦截内网目标"
    这两个开关默认开启。要代理到内网目标，需要显式关闭并自行评估风险。

## 预留与未生效字段 {#reserved}

以下字段可被解析，但当前版本未接入运行逻辑。**不要依赖它们产生效果**：

| 字段 | 说明 |
| --- | --- |
| `udp_relay_ipv6` | 为 IPv6 UDP 使用独立 socket。 |
| `dual_stack` | 监听套接字双栈。 |
| `task_negotiation_timeout` | 任务协商超时。 |
| `gc_interval` | UDP 分片回收间隔。 |
| `gc_lifetime` | UDP 分片保留时间。 |
| `max_external_packet_size` | 出站 UDP 最大包大小。 |

`[restful].maximum_clients_per_user` 与证书热重载同样未生效，见上文。

## 旧字段迁移 {#migration}

以下旧字段在解析时自动迁移，建议尽快改用新写法：

| 旧写法 | 新写法 |
| --- | --- |
| 顶层 `self_sign`、`certificate`、`private_key`、`auto_ssl`、`hostname`、`acme_email`、`alpn` | `[tls]` 中同名字段。 |
| 顶层 `congestion_control`、`max_idle_time`、`initial_window`、`send_window`、`receive_window`、`initial_mtu`、`min_mtu`、`gso`、`pmtu` | `[backend.quinn]` 及 `[backend.quinn.congestion_control]`。 |
| `[quic]` 表 | `[backend.quinn]`。 |
| `[camouflage]` 表 | `[masquerade]`（`reverse_proxy_url` → `upstream`；`reverse_proxy_hostname`、`request_timeout`、`skip_backend_tls_verify` 被丢弃）。 |
| 顶层 `restful_server` 地址 | `[restful].addr`，并自动启用 API。 |

当 `[quic]` 与扁平标量同时出现时，扁平标量优先。

## 完整示例 {#example}

```toml
log_level = "info"
server = "[::]:8443"
data_dir = "/var/lib/tuic"
zero_rtt_handshake = false
auth_timeout = "3s"
stream_timeout = "60s"

[log]
format = "text"
compact = true
log_file = "/var/log/tuic/server.log"
log_rotation = "daily"

[users]
"00000000-0000-4000-8000-000000000001" = "change-this-password"

[tls]
hostname = "tuic.example.com"
alpn = ["h3"]
certificate = "/etc/tuic/fullchain.pem"
private_key = "/etc/tuic/privatekey.pem"

[backend]
mode = "quinn"

[backend.quinn.congestion_control]
controller = "bbr"

[outbound.default]
type = "direct"
ip_mode = "v4first"

[[acl]]
addr = "private"
outbound = "reject"

[rules]
"MATCH,default"

[experimental]
drop_loopback = true
drop_private = true

[restful]
enabled = false
addr = "127.0.0.1:13471"
secret = ""
maximum_clients_per_user = 0
```
