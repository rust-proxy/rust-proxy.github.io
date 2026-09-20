# 服务端

`tuic-server` 是一个积极维护的 TUIC 协议服务端实现。它 fork 自原始 TUIC 项目，在保持协议简洁、低握手开销的同时，增加了 Docker 支持、自签证书、ACME 自动签发、证书热重载、ACL/路由、出站与管理 API 等生产可用能力。

本页介绍服务端的安装、启动与配置。客户端请见[客户端](client.md)；Docker 部署请见 [Docker](docker.md)；完整示例与字段说明见[配置生成器](/config-generator/?schema=tuic-server&mode=detail)。

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

完整且带注释的服务端配置示例由[配置生成器](/config-generator/?schema=tuic-server&mode=detail)的“配置详解”维护：选择证书模式、QUIC 后端、拥塞控制、出站与路由等分支即可查看对应的完整 YAML 结构，悬浮或聚焦任意一行可查看字段说明。

[查看服务端完整配置详解](/config-generator/?schema=tuic-server&mode=detail){ .md-button .md-button--primary }

生成器默认输出 TOML，也可切换为 JSON 或 YAML，并可在“配置生成”视图中填写参数后直接复制或下载。`rules`、`[dns]`、`[geodata]`、`[restful]`、`[masquerade]` 等段落同样可以在生成器中配置。

!!! note "预留字段"

    `udp_relay_ipv6`、`dual_stack`、`task_negotiation_timeout` 等预留字段尚未接入运行逻辑，生成器不会输出。

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
