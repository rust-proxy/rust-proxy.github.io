# 客户端

`tuic-client` 是 TUIC 协议的客户端实现，提供本地 SOCKS5 服务与 TCP/UDP 端口转发。它保持精简，只包含一个可用 TUIC 客户端所需的核心能力；如需 HTTP 入站、负载均衡等功能，可自行实现或选用其它实现。

本页介绍客户端的安装、启动与配置。服务端请见[服务端](server.md)；最小配置与验证步骤见[快速入门](getting-started.md)；成对的示例配置可在[配置生成器](/config-generator/)本地生成。

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

### 完整示例

```toml
# 日志级别
log_level = "info"

# 服务器地址（主机名:端口或 IP:端口；IPv6 写作 [地址]:端口）
server = "example.com:443"

# 用户 UUID 与密码
uuid = "00000000-0000-4000-8000-000000000001"
password = "change-this-password"

# 可选：出站连接的绑定 IP
# ip = "192.168.1.100"

# IP 栈偏好：v4first, v6first, v4only, v6only
ipstack_prefer = "v4first"

# UDP 中继模式：native 或 quic
udp_relay_mode = "native"

# 启用 0-RTT 握手
zero_rtt_handshake = false

# 连接超时
timeout = "8s"

# 心跳间隔
heartbeat = "3s"

# 垃圾回收间隔与保留时长
gc_interval = "3s"
gc_lifetime = "15s"

# 连接中断后自动重连
reconnect = true
# 首次重连退避；每次失败后翻倍
reconnect_initial_backoff = "500ms"
# 重连退避上限
reconnect_max_backoff = "30s"

# 按需连接：true 为懒加载（默认），false 为启动即连接、失败即退出
lazy = true

[tls]
# 可选：自定义证书路径
# certificates = ["/path/to/cert.pem"]
# ALPN 协议（例如 ["h3"]）
alpn = []
# 禁用 SNI（Server Name Indication）
disable_sni = false
# 可选：覆盖 SNI 主机名
# sni = "custom.example.com"
# 禁用系统原生证书库
disable_native_certs = false
# 跳过证书校验（不安全，仅用于测试）
skip_cert_verify = false

[backend]
# QUIC 后端：quinn
mode = "quinn"

[backend.quinn]
# 拥塞控制：cubic, new_reno, bbr, bbr3
[backend.quinn.congestion_control]
controller = "bbr"
# QUIC 发送窗口（字节）
send_window = 16777216
# QUIC 接收窗口（字节）
receive_window = 8388608
# 初始 MTU
initial_mtu = 1200
# 最小 MTU
min_mtu = 1200
# 启用通用分段卸载（GSO）
gso = true
# 启用路径 MTU 发现
pmtu = true

# 可选：通过上游 SOCKS5/HTTP 代理连接服务器
# [proxy]
# server = "127.0.0.1:1080"
# username = "proxy_user"
# password = "proxy_pass"
# udp_buffer_size = 2048

[local]
# 本地 SOCKS5 服务地址
server = "127.0.0.1:1080"
# 可选：SOCKS5 认证
# username = "socks_user"
# password = "socks_pass"
# 是否启用双栈（IPv4 与 IPv6）
dual_stack = true
# 最大 UDP 包大小
max_packet_size = 1500

# TCP 端口转发
# [[local.tcp_forward]]
# listen = "127.0.0.1:8080"
# remote = "example.com:80"

# UDP 端口转发
# [[local.udp_forward]]
# listen = "127.0.0.1:5353"
# remote = "8.8.8.8:53"
# timeout = "60s"
```

字段级说明与取值范围以[配置生成器](/config-generator/)为准。

### 端口转发

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
