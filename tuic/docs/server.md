# 服务端

`tuic-server` 是一个积极维护的 TUIC 协议服务端实现。它 fork 自原始 TUIC 项目，在保持协议简洁、低握手开销的同时，增加了 Docker 支持、自签证书、ACME 自动签发、证书热重载、ACL/路由、出站与管理 API 等生产可用能力。

本页介绍服务端的安装、启动与配置。客户端请见[客户端](client.md)；Docker 部署请见 [Docker](docker.md)；成对的示例配置可在[配置生成器](/config-generator/)本地生成。

## 安装

预编译的二进制文件可在 [GitHub Releases](https://github.com/Itsusinn/tuic/releases) 获取，也可以使用 Cargo 安装：

```console
cargo install --git https://github.com/Itsusinn/tuic.git tuic-server
```

从源码构建需要 Rust `1.85.0` 或更高版本以及 Git，详见[快速入门](getting-started.md)。

## 启动

```console
# 指定配置文件
tuic-server -c /etc/tuic/config.toml

# 指定配置目录：按字母序使用首个可识别的配置文件
tuic-server -d /etc/tuic

# 在当前目录生成示例配置 config.toml
tuic-server --init
```

`-d/--dir` 会在目录中查找首个可识别的配置文件（`.toml`、`.json`、`.json5`、`.yaml`、`.yml`），并按文件名排序。若未提供 `-c` 或 `-d`，服务器会报错退出。

## 配置

配置文件格式根据扩展名判断：

- **TOML**：`.toml`（推荐）
- **JSON5**：`.json`、`.json5`（兼容旧配置）
- **YAML**：`.yaml`、`.yml`

扩展名无法识别时服务器会报错退出；可用环境变量 `TUIC_CONFIG_FORMAT`（`toml`、`json`、`json5`、`yaml`、`yml`）显式指定格式。`[quic]`、`[camouflage]` 等旧写法会在解析时自动迁移，但新配置应使用下文的当前写法。

### 完整示例

```toml
# 日志级别：trace, debug, info, warn, error, off
log_level = "info"

# 监听地址
server = "[::]:8443"

# 工作目录（用于解析证书/私钥的相对路径）
data_dir = ""

# 为 IPv6 UDP 中继创建独立 socket
udp_relay_ipv6 = true
# 启用 0-RTT QUIC 握手（出于安全考虑建议 false）
zero_rtt_handshake = false
# 监听 socket 是否使用双栈（IPv4/IPv6）
dual_stack = true
# 等待客户端认证命令的最长时间
auth_timeout = "3s"
# 任务协商的最长时间
task_negotiation_timeout = "3s"
# UDP 分片垃圾回收间隔
gc_interval = "10s"
# UDP 分片保留时长
gc_lifetime = "30s"
# 出站 UDP socket 接收的最大包大小（字节）
max_external_packet_size = 1500
# TCP/UDP I/O 任务的保留时长
stream_timeout = "60s"

[log]
# 输出格式：text（默认）、json
format = "text"
# 紧凑格式（单行、更简洁），仅对 text 生效
compact = true
# 可选日志文件路径；设置后会同时写入该文件
# log_file = "/var/log/tuic/server.log"
# 日志轮转策略：never（默认）、hourly、daily
# log_rotation = "daily"

[users]
# 用户列表：UUID = 密码
"00000000-0000-4000-8000-000000000001" = "change-this-password"

[tls]
# 使用自动生成的自签证书与私钥
self_sign = false
# 证书路径（相对路径基于 data_dir）
certificate = ""
# 私钥路径（相对路径基于 data_dir）
private_key = ""
# ALPN 协议（例如 ["h3"]）
alpn = []
# 用于签发证书或自签的域名/IP
hostname = "localhost"
# 启用内置 ACME 自动申请证书
auto_ssl = false
# ACME 账户邮箱；留空时使用 admin@<hostname>
acme_email = ""
# 使用 Let's Encrypt 的 staging 环境（测试用）
acme_staging = false

[masquerade]
# 对非 TUIC 的 HTTP/3 探测流量启用反向代理伪装
enabled = false
# 反向代理上游站点
upstream = "https://example.com"

[backend]
# QUIC 后端：quinn（默认）或 quiche（实验性，需编译时启用 quiche feature）
mode = "quinn"

[backend.quinn]
# 拥塞控制：bbr, bbr3, cubic, new_reno
[backend.quinn.congestion_control]
controller = "bbr"
# 初始拥塞窗口（字节）
initial_window = 1048576
# MTU 发现前的初始 UDP 载荷
initial_mtu = 1200
# 网络保证支持的最小 UDP 载荷，须 ≥1200 且 ≤ initial_mtu
min_mtu = 1200
# 启用通用分段卸载（GSO）
gso = true
# 启用路径 MTU 发现
pmtu = true
# 未确认情况下允许发送的最大字节数
send_window = 16777216
# 对端每条流未确认情况下允许发送的最大字节数
receive_window = 8388608
# 空闲连接关闭前的等待时间
max_idle_time = "30s"

# Access Control List（ACL）：数组表格式
[[acl]]
# 地址：IPv4/IPv6、CIDR、域名、通配域名、localhost 或 private
addr = "127.0.0.1"
# 端口：逗号分隔，可带协议（如 "udp/53,tcp/80,udp/10000-20000,443"）
ports = "udp/53"
# 出站：direct / default / drop / <自定义出站名>
outbound = "default"
# 可选：重定向到指定地址
hijack = "1.1.1.1"

[outbound.default]
# 出站类型：direct 或 socks5
type = "direct"
# IP 模式：v4first, v6first, v4only, v6only
ip_mode = "v4first"

# 命名出站，可被 ACL 引用
[outbound.through_socks5]
type = "socks5"
addr = "127.0.0.1:1080"
# 可选 SOCKS5 认证
# username = "optional"
# password = "optional"
# 是否允许该出站的 UDP（默认 false；UDP 仍直连，尚未实现 SOCKS5 UDP 转发）
allow_udp = false

[experimental]
# 丢弃回环目标
drop_loopback = true
# 丢弃私有目标
drop_private = true

[restful]
# 是否启用 RESTful 管理 API
enabled = false
# 监听地址
addr = "127.0.0.1:13471"
# Bearer 令牌；为空表示不校验（不建议在公网暴露）
secret = "YOUR_SECRET_HERE"
# 每个用户的最大并发连接数（0 表示不限制）
maximum_clients_per_user = 0
```

此外还支持 `rules`（Metacubex 风格路由规则）、`[dns]`（DNS 解析）与 `[geodata]`（`geosite.dat` / `geoip.dat`）等段落。字段级说明与取值范围以[配置生成器](/config-generator/)为准。

### 出站与 ACL

- `[outbound.default]` 是默认出站；`[outbound.<name>]` 定义可在 ACL 中按名引用的出站。
- `direct` 出站支持 `bind_ipv4`/`bind_ipv6`（单个或多个地址，多个时按轮询负载均衡）、`bind_device`、`tfo`、`routing_mark`。
- ACL 规则按顺序匹配；未匹配的流量使用默认出站。
- 默认会丢弃回环与私有目标（`experimental.drop_loopback`、`experimental.drop_private`）；如需访问内网，请显式配置。

## TLS 与证书

TLS 是必需项，可通过以下方式之一提供证书。

### 内置 ACME

服务端内置 ACME，可通过 Let's Encrypt 自动申请并续期证书：

```toml
[tls]
auto_ssl = true
hostname = "your.domain.com"
acme_email = "admin@your.domain.com"
```

注意事项：

- 服务器必须能从公网访问 **TCP 80** 端口以完成 HTTP-01 验证。
- 在 Linux 上以非 root 用户运行时，可能需要允许绑定特权端口：
  ```sh
  setcap CAP_NET_BIND_SERVICE=+eip <tuic-server 可执行文件路径>
  ```
- 若 ACME 申请失败，会在配置了自签证书时回退到自签。

### 手动签发

也可以使用 [acme.sh](https://github.com/acmesh-official/acme.sh) 等工具手动获取证书：

```sh
acme.sh --issue -d www.yourdomain.org --standalone
acme.sh --install-cert -d www.yourdomain.org \
  --key-file       /CERT_PATH/key.crt  \
  --fullchain-file /CERT_PATH/cert.crt
```

随后在 `[tls]` 中填写 `certificate` 与 `private_key`，或改用 `self_sign = true` 生成自签证书（仅建议测试使用）。

## Docker

使用官方镜像运行服务端的完整步骤（`docker run`、Docker Compose、数据目录与证书挂载、排错）见 [Docker 部署](docker.md)。

## 许可证

本仓库代码依据 [GNU General Public License v3.0 or later](https://github.com/Itsusinn/tuic/blob/main/LICENSE) 发布。欢迎提交 Issue 与 Pull Request。
