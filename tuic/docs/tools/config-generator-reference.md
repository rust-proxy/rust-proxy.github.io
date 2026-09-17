# 配置生成器说明

[打开配置生成器](/config-generator/){ .md-button .md-button--primary }

生成器面向 **Itsusinn/tuic 2.0.0-dev7**，以 TUIC [`fd7e521`](https://github.com/Itsusinn/tuic/tree/fd7e521b54d637ba877a9a814b8757bfbe6a1e4f) 与其 Wind 子模块 [`dee0317`](https://github.com/rust-proxy/wind/tree/dee03178c0fcf230792cb9222e8125032e35a4dc) 为核对基线。它生成现代分组配置，采用 Quinn 后端。

## 使用流程

1. 选择配对生成、仅服务端或仅客户端。
2. 填写连接与认证信息。首次打开时生成随机 UUID 和密码；连接已有服务端时替换为已有凭据。
3. 配置 TLS，并按需设置本地代理、日志、连接、端口转发、QUIC 后端、路由与管理选项。
4. 修正标记的字段，选择 TOML、JSON 或 YAML，复制或下载配置。配对模式需要分别保存服务端和客户端文件。
5. 将证书放到配置指定的部署路径，开放 UDP 端口，再使用预览下方的命令启动对应程序。

页面顶部的“配置详解”提供服务端与客户端的完整 YAML 结构。左侧只切换证书方式、拥塞控制、路由、地址形式、SOCKS5 认证和端口转发等会改变配置结构的分支；0-RTT、重连和证书验证等普通布尔开关直接显示在 YAML 中。右侧示例会随结构分支同步变化。将鼠标悬浮在任意配置行上，或使用键盘聚焦该行，可以查看字段作用与安全注意事项。详解中的凭据和地址是占位示例，部署时仍应通过“配置生成”创建实际配置。

生成器是 Rust WASM + Svelte 编写的独立单页应用，需要支持 JavaScript 和 WebAssembly 的现代浏览器。所有输入与序列化都在浏览器本地进行，不写入 URL 或浏览器存储。页面使用自身的样式和主题，不加载文档站框架或分析脚本。刷新或离开页面会丢失输入。预览默认隐藏密码，**复制与下载包含明文凭据**，应妥善保存。

表单校验通过只表示字段符合生成器的约束，不代表 DNS、证书文件、防火墙或代理连接已经验证。

## 覆盖范围

服务端输出：`server`、`log_level`、`[log]`、`data_dir`、`users`、`[tls]`（含 ACME 与预发布环境）、`[backend]`（`quinn` 全量传输参数与实验性 `quiche`）、`zero_rtt_handshake`、`auth_timeout`、`stream_timeout`、`[outbound]`（default 与具名出站、direct/socks5）、`[acl]`、`rules`、`[dns]`、`[geodata]`、`[experimental]`、`[restful]`、`[masquerade]`。

客户端输出：`server`、`ip`、`uuid`、`password`、`log_level`、`udp_relay_mode`、`zero_rtt_handshake`、`heartbeat`、`gc_interval`、`gc_lifetime`、`reconnect`、`reconnect_initial_backoff`、`reconnect_max_backoff`、`lazy`、`[tls]`、`[backend.quinn]`、`[local]`（SOCKS5 与 TCP/UDP 转发）。

完整字段说明见[服务端配置](../server/config.md)与[客户端配置](../client/config.md)。

## 连接地址 {#addresses}

| 表单 | 输出字段 | 含义 |
| --- | --- | --- |
| 连接域名或 IP、端口 | 客户端 `server` | 从客户端可达的服务器地址 |
| 服务端监听地址 | 服务端 `server` | 服务端本机的 IP 和 UDP 端口 |
| 客户端 SNI | 客户端 `tls.sni` | 与证书相符的 DNS 域名 |

监听地址默认为 `[::]:8443`，客户端连接端口默认为 `8443`。二者独立设置，支持容器或 NAT 端口映射。监听地址只接受 IP；IPv6 地址与端口组合时使用 `[IPv6]:端口`。

连接地址可填域名、IPv4 或 IPv6。使用 IP 连接时仍需填写证书域名作为 SNI。生成器会同时写入客户端的 `ip` 字段，避免此版本对 IPv6 字面量执行 DNS 地址拼接。

## 用户认证 {#users}

服务端 `users` 是 UUID 到密码的映射；客户端使用顶层 `uuid` 与 `password`。配对模式从服务端用户列表中选择一个用户用于客户端配置。新增、删除、切换用户后，两份输出自动更新。

生成按钮使用浏览器加密随机源，生成 UUID v4 与 24 字节随机值的十六进制密码。需要 HTTPS 或 localhost。若浏览器不支持，可手动输入。生成器拒绝空密码、全零 UUID 和重复 UUID，不修改密码中的空格或特殊字符。

## TLS 与证书 {#tls}

| 方式 | 服务端输出 | 客户端行为 |
| --- | --- | --- |
| 已有受信任证书 | `tls.certificate`、`tls.private_key`、`tls.hostname` | 默认验证系统信任链与服务端身份 |
| ACME | `tls.auto_ssl`、`tls.acme_email`、`tls.hostname`、`tls.acme_staging`、`data_dir` | 默认验证证书 |
| 自签名测试 | `tls.self_sign`、`tls.hostname` | 配对测试需明确启用 `tls.skip_cert_verify` |

证书域名留空时沿用连接域名。仅生成服务端或以 IP 连接时，应明确填写。证书与私钥仅填写部署机器上的路径，生成器不创建或上传证书。

两端均显式输出 `tls.alpn = ["h3"]`，确保协议协商一致。此基线服务端的默认 ALPN 列表为空，客户端默认回退到 `h3`，只省略配置会导致连接失败。连接已有服务端时，请确认它也配置了 `h3`。

ACME 使用公开 DNS 域名；此基线通过 HTTP-01 在 **TCP 80** 完成验证。域名应解析到该服务器，端口可达且没有冲突。`data_dir` 用于缓存，需可写并持久化。TUIC 流量仍使用配置的 UDP 端口。

切换服务端证书方式会关闭“跳过证书校验”，并从输出移除先前模式的专用字段。自签名模式不会自动削弱客户端验证；需要手动勾选才能导出配对测试配置。

## 本地 SOCKS5 {#local}

`local.server` 默认为 `127.0.0.1:1080`。开启本地认证后同时输出 `local.username` 和 `local.password`，每项最多 255 个 UTF-8 字节。该认证与 TUIC 用户认证独立。

改为非回环监听地址时，生成器会提示未设置认证的情况。实际访问范围仍需通过本机防火墙控制。

## 日志、连接与传输 {#transport}

| 选项 | 默认值 | 输出与限制 |
| --- | --- | --- |
| 日志 | `info` | 两端 `log_level` |
| 文件日志 | 关闭 | 开启后服务端输出完整 `[log]` 表 |
| 0-RTT | 关闭 | 开启后写入 `zero_rtt_handshake`；早期数据可能重放 |
| 按需连接 | 开启 | 客户端 `lazy`，首次请求才连接 |
| 自动重连 | 开启 | 客户端 `reconnect` 与退避时间 |
| 认证 / 流超时 | 3 / 60 秒 | 服务端 `auth_timeout`、`stream_timeout` |
| UDP 转发模式 | `native` | 客户端 `udp_relay_mode` |
| 心跳 / 分片回收 | 3 / 3 / 15 秒 | 客户端 `heartbeat`、`gc_interval`、`gc_lifetime` |

服务端支持 `bbr`、`bbr3`、`cubic`、`newreno`。注意序列化值是 `newreno`；此基线的 `bbr` 和 `bbr3` 分支共用同一 BBR 工厂，不能据此承诺两者有不同性能。

## QUIC 后端 {#backend}

服务端可选择 `quinn`（默认）或实验性 `quiche`。Quinn 后端可配置拥塞控制、初始拥塞窗口、MTU、GSO、PMTU、收发窗口和空闲超时；quiche 后端可配置拥塞控制、空闲超时、并发流、收发窗口和 0-RTT。客户端仅支持 Quinn，并可配置拥塞控制与收发窗口。

## 路由与出站 {#routing}

- **出站**：至少保留一个名为 `default` 的出站。`direct` 支持地址族偏好、绑定地址与网卡、TCP Fast Open 和路由标记；`socks5` 支持地址、凭据和 UDP 放行。绑定多个地址需要手动改写为数组。
- **ACL**：`[[acl]]` 表，字段为 `addr`、`outbound`、可选的 `ports` 与 `hijack`。地址支持 IP、CIDR、域名、`localhost`、`private`、`*`。
- **路由规则**：Metacubex 风格字符串，在 ACL 之后求值。
- **DNS**：`system` 或预设解析器，或 `custom` 下的自定义服务器列表。
- **GeoData**：`geosite`/`geoip` 数据库路径，`GEOIP`/`GEOSITE` 规则需要它。

`hijack` 字段会被写出，但当前版本尚未执行重定向。

## 管理与伪装 {#management}

- **实验性**：`drop_loopback`、`drop_private`，默认拦截回环与私有目标。
- **RESTful**：可启用管理 API、设置监听地址、Bearer 令牌与每用户并发上限。`maximum_clients_per_user` 当前未强制生效。
- **伪装**：`masquerade` 把非 TUIC 的 HTTP/3 连接反向代理到真实网站。

## 当前限制 {#limitations}

- 仅输出此版本的现代配置；不导入旧 `[relay]`、`[camouflage]`、`[quic]` 等旧字段，不生成第三方客户端格式。
- 预留字段（服务端 `udp_relay_ipv6`、`dual_stack`、`task_negotiation_timeout`、`gc_*`、`max_external_packet_size`；客户端 `ipstack_prefer`、`timeout`、`proxy`、`tls.certificates`、`tls.disable_sni`、`tls.disable_native_certs`、MTU/GSO/PMTU、`local.dual_stack`、`local.max_packet_size`）当前未接入运行逻辑，生成器不输出。
- 不提供 quiche 后端的构建开关、证书申请服务或浏览器内连接测试。
- 不尝试在浏览器中连接 TUIC 服务器或验证远程文件路径。

字段与行为依据：[客户端配置](https://github.com/Itsusinn/tuic/blob/fd7e521b54d637ba877a9a814b8757bfbe6a1e4f/crates/tuic-client/src/config.rs)、[服务端配置](https://github.com/Itsusinn/tuic/blob/fd7e521b54d637ba877a9a814b8757bfbe6a1e4f/crates/tuic-server/src/config.rs)。
