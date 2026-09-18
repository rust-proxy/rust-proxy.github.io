# 快速开始

本页用最短步骤跑通一对 `tuic-server` 与 `tuic-client`。

## 前提

- 一台可被客户端访问的服务器，具有公网 IP 或可解析的域名。
- 服务器放行 **UDP** 监听端口；代理 TCP 流量同样走 UDP，QUIC 没有 TCP 监听端口。
- 客户端与服务器时间大致同步，能解析彼此的地址。

## 获取程序

预编译二进制见 [GitHub Releases](https://github.com/Itsusinn/tuic/releases)。也可以用 Cargo 安装：

```console
cargo install --git https://github.com/Itsusinn/tuic.git tuic-server
cargo install --git https://github.com/Itsusinn/tuic.git tuic-client
```

从源码构建需要 Rust `1.85.0` 或更高版本和 Git。仓库包含 submodule，克隆时一并初始化：

```console
git clone --recurse-submodules https://github.com/Itsusinn/tuic.git
cd tuic
cargo build --release --package tuic-server --package tuic-client
```

产物位于 `target/release/tuic-server` 与 `target/release/tuic-client`。

## 服务端

### 生成示例配置

```console
tuic-server --init
```

该命令在当前目录生成 `config.toml`，包含 5 个随机用户；若文件已存在则拒绝覆盖。也可以从下面的最小配置开始。

### 最小配置

```toml
log_level = "info"
server = "[::]:8443"

[users]
"00000000-0000-4000-8000-000000000001" = "change-this-password"

[tls]
self_sign = true
hostname = "tuic.example.com"
alpn = ["h3"]
```

### 启动

```console
# 指定配置文件
tuic-server -c /etc/tuic/config.toml

# 或指定目录，自动使用其中按字母序第一个可识别的配置文件
tuic-server -d /etc/tuic
```

未提供 `-c` 或 `-d` 时服务端会报错退出。配置文件格式由扩展名推断：`.toml`、`.json`、`.json5`、`.yaml`、`.yml`。

### 防火墙

至少放行 UDP 监听端口（示例为 `8443`）。ACME 自动证书还需要放行 **TCP 80** 用于 HTTP-01 验证。

### Docker

```console
docker run --name tuic-server \
  --restart always \
  --network host \
  -v /PATH/TO/DATA_DIR:/var/lib/tuic/ \
  -v /PATH/TO/CONFIG_FILE:/etc/tuic/config.toml \
  -v /PATH/TO/CERTIFICATE:/var/lib/tuic/fullchain.pem \
  -v /PATH/TO/PRIVATE_KEY:/var/lib/tuic/key.pem \
  -dit ghcr.io/itsusinn/tuic-server:latest
```

容器会使用 `/etc/tuic` 下按字母序第一个配置文件。

## 客户端

### 最小配置

```toml
log_level = "info"
server = "tuic.example.com:8443"
uuid = "00000000-0000-4000-8000-000000000001"
password = "change-this-password"

[tls]
sni = "tuic.example.com"
alpn = ["h3"]

[local]
server = "127.0.0.1:1080"
```

- `server` 是客户端可达的 `host:port`，IPv6 必须写作 `[addr]:port`。
- 用 IP 连接时仍应把证书域名填入 `tls.sni`，否则 TLS 校验会失败。
- 若服务端使用自签名证书，测试时需加 `skip_cert_verify = true`（生产环境不应使用）。

### 启动

```console
tuic-client -c /etc/tuic/client.toml
```

客户端默认按需连接（`lazy = true`）：收到第一个代理请求时才建立 QUIC 连接。连接断开后默认自动重连。

### 让应用使用代理

客户端在 `local.server` 暴露一个 SOCKS5 服务。将应用或系统的 SOCKS5 代理指向 `127.0.0.1:1080` 即可。例如：

```console
curl --socks5-hostname 127.0.0.1:1080 https://example.com
```

`--socks5-hostname` 让域名在服务端解析，适合需要远端 DNS 的场景。

### 端口转发

除 SOCKS5 外，客户端可把本地端口直接转发到远端目标：

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

服务端默认拦截回环和私有目标地址（`experimental.drop_loopback`、`experimental.drop_private`）。访问内网目标需要另行调整服务端路由与访问控制。

## 用配置生成器起步

[打开配置生成器](/config-generator/){ .md-button .md-button--primary }

生成器在浏览器本地运行，可配对生成服务端与客户端配置，支持多用户、TLS、SOCKS5 认证和端口转发，并输出 TOML、JSON 或 YAML。

## 验证

1. 服务端日志出现监听与 TLS 就绪信息。
2. 客户端启动后，通过本地 SOCKS5 请求一个外部地址。
3. 若失败，依次检查：UDP 端口是否放行、`tls.sni` 是否与证书匹配、两端 `uuid`/`password` 是否一致、`tls.alpn` 是否一致。

配置校验通过只代表字段合法，不代表 DNS、证书、防火墙或实际链路已经验证。
