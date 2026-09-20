# 快速入门

本页面提供了让 `tuic-server` 和 `tuic-client` 组合快速投入运行的最简步骤。

本页只覆盖最小可用步骤；完整的服务端与客户端配置见[服务端](server.md)与[客户端](client.md)。

## 先决条件

- 客户端可访问的服务器，该服务器需拥有公共 IP 地址或可解析的域名。
- 服务器必须允许 **UDP** 监听端口；TCP 流量也会通过 UDP 进行代理，且 QUIC 不使用 TCP 监听端口。
- 客户端和服务器的时间大致同步，并且能够解析彼此的地址。

## 获取软件

预编译的二进制文件可在 [GitHub Releases](https://github.com/Itsusinn/tuic/releases) 上获取。您也可以使用 Cargo 进行安装：

```console
cargo install --git https://github.com/Itsusinn/tuic.git tuic-server
cargo install --git https://github.com/Itsusinn/tuic.git tuic-client
```

从源代码构建需要 Rust `1.85.0` 或更高版本以及 Git。该代码库包含子模块，这些子模块会在克隆过程中自动初始化：

```console
git clone --recurse-submodules https://github.com/Itsusinn/tuic.git
cd tuic
cargo build --release --package tuic-server --package tuic-client
```

构建生成的文件位于 `target/release/tuic-server` 和 `target/release/tuic-client` 目录中。

## 服务器

### 生成示例配置

```console
tuic-server --init
```

此命令会在当前目录中生成一个包含 5 个随机用户的 `config.toml` 文件；如果该文件已存在，则不会覆盖它。您也可以从下面的最小配置开始。

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

# 或指定一个目录；系统将自动使用按字母顺序排列的首个可识别配置文件
tuic-server -d /etc/tuic
```

如果未提供 `-c` 或 `-d` 选项，服务器将报告错误并退出。配置文件的格式根据文件扩展名推断：`.toml`、`.json`、`.json5`、`.yaml` 或 `.yml`。

### 防火墙

至少需允许 UDP 监听端口（例如 `8443`）上的流量通过。ACME 自动证书还要求 **TCP 端口 80** 处于开放状态，以供 HTTP-01 验证使用。


## 客户端

### 最低配置

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

- `server` 是客户端可访问的 `主机:端口` 地址；对于 IPv6，必须写为 `[地址]:端口`。
- 通过 IP 连接时，仍应在 `tls.sni` 中指定证书域名；否则，TLS 验证将失败。
- 如果服务器使用自签名证书，在测试期间请添加 `skip_cert_verify = true`（生产环境中不应使用此设置）。

### 启动客户端

```console
tuic-client -c /etc/tuic/client.toml
```

默认情况下，客户端按需连接（`lazy = true`）：仅在收到第一个代理请求时才建立 QUIC 连接。连接中断后，默认会自动重新连接。

### 配置应用程序以使用代理

客户端在 `local.server` 暴露一个 SOCKS5 服务。只需将应用程序或系统的 SOCKS5 代理指向 `127.0.0.1:1080` 即可。例如：

```console
curl --socks5-hostname 127.0.0.1:1080 https://example.com
```

`--socks5-hostname` 选项会在服务器端解析域名，这适用于需要远程 DNS 的场景。

### 端口转发

除了 SOCKS5 之外，客户端还可以将本地端口直接转发到远程目标：

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

默认情况下，服务器会阻止回环地址和私有目标地址（`experimental.drop_loopback`、`experimental.drop_private`）。若要访问内部网络目标，您必须单独配置服务器的路由和访问控制。

## 开始使用配置生成器

[打开配置生成器](/config-generator/){ .md-button .md-button--primary }

该生成器在您的浏览器中本地运行，通过顶部的配置方案选择器分别生成服务端与客户端配置。它支持多用户、TLS、SOCKS5 身份验证、出站与路由以及端口转发，并输出 TOML、JSON 或 YAML 格式。需要带注释的完整示例时，可直接打开[服务端配置详解](/config-generator/?schema=tuic-server&mode=detail)或[客户端配置详解](/config-generator/?schema=tuic-client&mode=detail)。

## 验证

1. 服务器日志应显示其正在监听并已准备好进行 TLS 连接。
2. 启动客户端后，从本地机器向外部地址发送一个 SOCKS5 请求。
3. 如果请求失败，请按以下顺序检查：UDP 端口是否已打开、`tls.sni` 是否与证书匹配、双方的 `uuid` 和 `password` 是否一致，以及 `tls.alpn` 是否匹配。

通过配置验证仅表示字段有效；并不意味着 DNS、证书、防火墙或实际网络连接已通过验证。

