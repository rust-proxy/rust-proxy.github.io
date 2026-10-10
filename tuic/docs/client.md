# 客户端

`tuic-client` 是 TUIC 协议的客户端实现，提供本地 SOCKS5 服务与 TCP/UDP 端口转发。它保持精简，只包含一个可用 TUIC 客户端所需的核心能力；如需 HTTP 入站、负载均衡等功能，可自行实现或选用其它实现。

本页介绍客户端的安装、启动与配置。服务端请见[服务端](server.md)；最小配置与验证步骤见[快速入门](getting-started.md)；完整示例与字段说明见[配置编辑器](/config-editor/?schema=tuic-client)。

## 安装

预编译的二进制文件可在 [GitHub Releases](https://github.com/Itsusinn/tuic/releases) 获取，也可以使用 Cargo 安装：

```console
cargo install --git https://github.com/Itsusinn/tuic.git tuic-client
```

从源码构建需要 Rust `1.85.0` 或更高版本以及 Git，详见[快速入门](getting-started.md)。

## 启动

```console
tuic-client -c /etc/tuic/client.toml
```

默认按需连接（`lazy = true`）：仅在收到第一个代理请求时才建立 QUIC 连接；连接中断后默认自动重连。客户端在 `local.server` 暴露 SOCKS5 服务，将应用指向该地址即可，例如：

```console
curl --socks5-hostname 127.0.0.1:1080 https://example.com
```

## 配置

配置文件格式根据扩展名判断：

- **TOML**：`.toml`（推荐）
- **JSON5**：`.json`、`.json5`（兼容旧配置）
- **YAML**：`.yaml`、`.yml`

扩展名无法识别时客户端会报错退出；可用环境变量 `TUIC_CONFIG_FORMAT`（`toml`、`json`、`json5`、`yaml`、`yml`）显式指定格式。旧的 `[relay]` 段会在解析时自动迁移到顶层连接字段与 `[tls]`，但新配置应使用下文的当前写法。

其中旧 `[relay]` 的传输调优字段按 `[backend] mode` 迁移：`congestion_control`、`send_window`、`receive_window` 会写入实际选中的后端（`[backend.quinn]` 或 `[backend.quiche]`）；`initial_mtu`、`min_mtu`、`gso`、`pmtu` 在 quiche 后端下没有对应字段，仍保留在 `[backend.quinn]`，并会输出一条点名这些键的警告。

### TLS 证书与 SNI

`[tls]` 下的三个开关由 quinn 后端（默认后端）实现，作用如下：

- `disable_sni = true`：不在 ClientHello 中发送 SNI 扩展，主机名不再泄露给中间人；证书仍按 `sni`（未设置时按 `server` 的主机名）校验。若 `server` 是 IP 字面量且未设置 `sni`，请改用 `sni` 指定证书中的名称。
- `disable_native_certs = true`：不使用平台信任库，改用内置的 Mozilla 根证书集合；配合 `certificates` 可只信任自建 CA。
- `certificates = ["/path/to/ca.pem"]`：追加信任的 CA 证书文件（PEM，可含多个证书），用于校验自签或私有 CA 签发的服务端证书。文件不存在或解析不出证书时客户端会在启动时直接报错。

`skip_cert_verify = true` 仍然优先于上述配置：它完全不校验证书，仅供测试或明确可信的环境使用。

`[backend] mode = "quiche"` 时这三个 TLS 开关暂无对应实现：设置它们会输出一条警告说明其在 quiche 后端不生效，而不是静默忽略。

## 完整示例

完整且带注释的客户端配置由[配置编辑器](/config-editor/?schema=tuic-client)的预览区维护：在左侧表单填写参数，右侧实时生成配置，逐行悬浮或聚焦可查看字段说明，并同步显示校验结果与复制、下载入口。

[打开客户端配置编辑器](/config-editor/?schema=tuic-client){ .md-button .md-button--primary }

编辑器默认输出 TOML，也可切换为 JSON 或 YAML，并可直接复制或下载。编辑器暂不生成 `ipstack_prefer`、`timeout`、`[proxy]`、`dual_stack`；上游代理可以按下节说明手动加入配置。

## SOCKS5 上游代理 {#socks5-upstream}

在现有客户端配置中添加 `[proxy]`，可让 TUIC 的 QUIC 连接经过 SOCKS5 上游。Quinn 和 Quiche 后端均支持，上游必须提供 **UDP ASSOCIATE**；仅支持 TCP CONNECT 的 SOCKS 服务无法承载 QUIC。

```toml
# 以下字段位于配置顶层，放在其他 [section] 之前。
server = "tuic.example:124"
timeout = "8s"
reconnect = true
reconnect_initial_backoff = "500ms"
reconnect_max_backoff = "30s"

[proxy]
server = "socks.example:123"
udp_buffer_size = 2048
# 需要认证时，同时填写 username 和 password。
```

示例是配置片段，需保留原配置中的 TUIC 身份凭据与 TLS 设置。代理用户名和密码必须同时设置，各包含 1–255 字节；省略时使用无认证方式。`udp_buffer_size` 是有界分配提示，较小的值不会截断 QUIC 报文。

代理模式下，TUIC 服务器域名交给 SOCKS 上游解析，不会在客户端解析；若顶层显式设置 `ip`，则上游直接连接该 IP。SOCKS 服务器自身的域名仍由客户端解析。未配置代理时，服务器解析行为保持不变。

客户端自动创建名为 `__GENERATED_CHAINPROXY__tuic` 的本地 UDP Tunnel，绑定系统分配的回环端口，固定转发到 `SOCKS_UPSTREAM`。TUIC 后端连接此 Tunnel，但 SNI、证书校验名称和认证信息仍来自原服务器配置。内部端口和名称不会写回配置文件。

`lazy = true` 时首次业务流量才连接上游；`lazy = false` 时启动期间完成握手，失败会使启动失败。`timeout` 限制上游会话建立和代理模式下的初次 TUIC 握手。代理控制连接断开后按重连参数恢复；关闭重连时对应链路停止。任何代理故障都不会回退到直连 TUIC 服务器。

## 端口转发

除 SOCKS5 外，客户端还可以把本地端口直接转发到远端目标：

```toml
[local]
server = "127.0.0.1:1080"

[[local.tcp_forward]]
listen = "127.0.0.1:8080"
remote = "example.com:80"

[[local.udp_forward]]
listen = "127.0.0.1:5353"
remote = "8.8.8.8:53"
timeout = "60s"
```

默认情况下，服务端会阻止回环与私有目标；如需访问内网，需在服务端配置路由与访问控制。

## 许可证

本仓库代码依据 [GNU General Public License v3.0 or later](https://github.com/Itsusinn/tuic/blob/main/LICENSE) 发布。欢迎提交 Issue 与 Pull Request。
