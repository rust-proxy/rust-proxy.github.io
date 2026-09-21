# 客户端

`tuic-client` 是 TUIC 协议的客户端实现，提供本地 SOCKS5 服务与 TCP/UDP 端口转发。它保持精简，只包含一个可用 TUIC 客户端所需的核心能力；如需 HTTP 入站、负载均衡等功能，可自行实现或选用其它实现。

本页介绍客户端的安装、启动与配置。服务端请见[服务端](server.md)；最小配置与验证步骤见[快速入门](getting-started.md)；完整示例与字段说明见[配置生成器](/config-generator/?schema=tuic-client)。

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

完整且带注释的客户端配置由[配置生成器](/config-generator/?schema=tuic-client)的预览区维护：在左侧表单填写参数，右侧实时生成配置，逐行悬浮或聚焦可查看字段说明，并同步显示校验结果与复制、下载入口。

[打开客户端配置生成器](/config-generator/?schema=tuic-client){ .md-button .md-button--primary }

生成器默认输出 TOML，也可切换为 JSON 或 YAML，并可直接复制或下载。它只输出已接入运行逻辑的字段；`ipstack_prefer`、`timeout`、`[proxy]`、`dual_stack` 等预留字段不会被生成。

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
