# 客户端配置

`tuic-client` 读取单个配置文件，字段以 TOML、JSON5 或 YAML 表示。本页列出 2.0.0-dev7 的全部字段、默认值与行为；**未生效字段**单独标注。

## 配置文件与加载 {#format}

- 格式由扩展名推断：`.toml`、`.json`、`.json5`、`.yaml`、`.yml`。
- `TUIC_FORCE_TOML` 强制按 TOML 解析；`TUIC_CONFIG_FORMAT` 显式指定格式。
- 命令行只有 `-c`、`--config <PATH>`；未提供时报错退出。

现代布局把连接字段放在顶层，`[tls]`、`[backend.quinn]`、`[local]` 分组。旧的 `[relay]` 表仍被接受并自动迁移（见[旧 `[relay]` 迁移](#migration)），但会输出弃用告警。

## 顶层字段 {#top-level}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `server` | 字符串 | 必填 | 服务端 `host:port`；IPv6 使用 `[addr]:port`。 |
| `uuid` | UUID | 全零 | TUIC 用户 UUID。 |
| `password` | 字符串 | 空 | TUIC 用户密码。 |
| `ip` | IP | 无 | 显式服务端 IP，绕过域名解析。 |
| `udp_relay_mode` | 枚举 | `native` | `native` 或 `quic`。 |
| `zero_rtt_handshake` | 布尔 | `false` | 恢复会话时发送 0-RTT 早期数据。 |
| `heartbeat` | 时长 | `3s` | 心跳间隔。 |
| `gc_interval` | 时长 | `3s` | UDP 分片回收间隔。 |
| `gc_lifetime` | 时长 | `15s` | UDP 分片保留时间。 |
| `reconnect` | 布尔 | `true` | 断开后自动重连。 |
| `reconnect_initial_backoff` | 时长 | `500ms` | 首次重连等待。 |
| `reconnect_max_backoff` | 时长 | `30s` | 退避上限。 |
| `lazy` | 布尔 | `true` | 按需连接；`false` 为启动即连接，失败时退出。 |
| `log_level` | 字符串 | `info` | 日志级别。 |
| `tls` | 表 | 见 [`[tls]`](#tls) | TLS 验证与协商。 |
| `backend` | 表 | 见 [`[backend.quinn]`](#backend) | QUIC 传输参数。 |
| `local` | 表 | 见 [`[local]`](#local) | 本地代理与转发。 |
| `proxy` | 表 | 见[预留字段](#reserved) | 上游 SOCKS5，当前未生效。 |
| `ipstack_prefer` | 枚举 | `v4first` | 地址族偏好，当前未生效。 |
| `timeout` | 时长 | `8s` | 连接超时，当前未生效。 |

## `[tls]` {#tls}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `sni` | 字符串 | 无 | SNI 覆盖；缺省时从 `server` 主机名推导。 |
| `alpn` | 字符串列表 | `[]` | ALPN 列表；为空时回退到 `h3`。 |
| `skip_cert_verify` | 布尔 | `false` | 跳过证书链与主机名校验。 |
| `certificates` | 路径列表 | `[]` | 自定义信任证书，当前未生效。 |
| `disable_sni` | 布尔 | `false` | 不发送 SNI，当前未生效。 |
| `disable_native_certs` | 布尔 | `false` | 禁用平台信任库，当前未生效。 |

用 IP 连接时，`server` 的主机名是 IP 字面量，无法作为 SNI。此时应显式设置 `tls.sni` 为证书域名；否则客户端会使用占位 SNI 并告警。

客户端默认使用平台证书验证（`rustls-platform-verifier`）。`tls.certificates` 不会被读取，因此自签名或私有 CA 场景目前只能通过 `skip_cert_verify = true` 绕过（仅限测试）。

!!! warning "ALPN 必须与服务端一致"
    服务端为空时不声明 ALPN；客户端为空时回退 `h3`。两端都建议显式设置 `alpn = ["h3"]`。

## `[backend.quinn]` {#backend}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `congestion_control.controller` | 枚举 | `bbr` | `bbr`、`bbr3`、`cubic`、`newreno`。 |
| `send_window` | 整数 | `16777216` | 未经确认可发送的最大字节数。 |
| `receive_window` | 整数 | `8388608` | 每条流可接收的最大字节数。 |
| `initial_mtu` | 整数 | `1200` | 初始 MTU，当前未生效。 |
| `min_mtu` | 整数 | `1200` | 最小 MTU，当前未生效。 |
| `gso` | 布尔 | `true` | UDP GSO，当前未生效。 |
| `pmtu` | 布尔 | `true` | 路径 MTU 发现，当前未生效。 |

客户端 `backend.mode` 仅支持 `quinn`。

## `[local]` {#local}

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `server` | 套接字 | `127.0.0.1:1080` | 本地 SOCKS5 监听地址。 |
| `username` | 字符串 | 无 | SOCKS5 用户名。 |
| `password` | 字符串 | 无 | SOCKS5 密码。 |
| `tcp_forward` | 表数组 | `[]` | 静态 TCP 转发。 |
| `udp_forward` | 表数组 | `[]` | 静态 UDP 转发。 |
| `dual_stack` | 布尔 | 无 | 双栈监听，当前未生效。 |
| `max_packet_size` | 整数 | `1500` | UDP 最大包大小，当前未生效。 |

SOCKS5 认证与 TUIC 用户认证相互独立。改为非回环监听地址时建议启用认证并配置防火墙。

转发项：

```toml
[[local.tcp_forward]]
listen = "127.0.0.1:8080"
remote = "example.com:80"

[[local.udp_forward]]
listen = "127.0.0.1:5353"
remote = "8.8.8.8:53"
timeout = "60s"
```

`listen` 必须是 `IP:端口`，`remote` 可为域名或 `IP:端口`。服务端默认拦截回环和私有目标地址，见[服务端 `[experimental]`](../server/config.md#experimental)。

## 预留与未生效字段 {#reserved}

以下字段可被解析，但当前版本未接入运行逻辑：

| 字段 | 说明 |
| --- | --- |
| `ipstack_prefer` | 地址族偏好。 |
| `timeout` | 连接超时。 |
| `proxy.server` / `proxy.username` / `proxy.password` / `proxy.udp_buffer_size` | 通过上游 SOCKS5 建立 QUIC 连接。 |
| `tls.certificates` | 自定义信任证书。 |
| `tls.disable_sni` | 抑制 SNI。 |
| `tls.disable_native_certs` | 禁用平台信任库。 |
| `backend.quinn.initial_mtu` / `min_mtu` / `gso` / `pmtu` | 传输细节。 |
| `local.dual_stack` / `max_packet_size` | 本地监听细节。 |

## 旧 `[relay]` 迁移 {#migration}

旧配置把所有连接字段放在 `[relay]` 下。解析时按下列规则迁移，且**现代字段优先**：

| `[relay]` 字段 | 迁移目标 |
| --- | --- |
| `certificates`、`alpn`、`disable_sni`、`sni`、`disable_native_certs`、`skip_cert_verify` | `[tls]` |
| `congestion_control` | `[backend.quinn.congestion_control].controller` |
| `send_window`、`receive_window`、`initial_mtu`、`min_mtu`、`gso`、`pmtu` | `[backend.quinn]` |
| 其余（`server`、`uuid`、`password`、`ip`、`udp_relay_mode` 等） | 顶层同名字段 |

迁移会输出弃用告警，建议尽快改写为现代布局。

## 完整示例 {#example}

```toml
log_level = "info"
server = "tuic.example.com:8443"
uuid = "00000000-0000-4000-8000-000000000001"
password = "change-this-password"
udp_relay_mode = "native"
zero_rtt_handshake = false
heartbeat = "3s"
reconnect = true
reconnect_initial_backoff = "500ms"
reconnect_max_backoff = "30s"
lazy = true

[tls]
sni = "tuic.example.com"
alpn = ["h3"]
skip_cert_verify = false

[backend.quinn.congestion_control]
controller = "bbr"

[local]
server = "127.0.0.1:1080"

[[local.tcp_forward]]
listen = "127.0.0.1:8080"
remote = "example.com:80"
```
