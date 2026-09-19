# Docker 部署

本页面介绍如何使用官方镜像运行 `tuic-server`。客户端仍按[快速入门](getting-started.md)在宿主机或其它容器中运行。

## 前置条件

- 已安装 Docker；使用 Compose 时需要 Docker Compose 插件。
- 服务器必须允许 **UDP** 监听端口（默认示例为 `8443`）。
- 一个用于存放 `config.toml` 的宿主机目录，例如 `/etc/tuic`。

## 镜像

镜像由 CI 从 `.github/Dockerfile` 构建并发布到 GitHub Container Registry：

```console
docker pull ghcr.io/itsusinn/tuic-server:latest
```

- 同时提供 `linux/amd64` 与 `linux/arm64` 两种架构。
- `latest` 指向最近一次发布；也可以使用对应的版本标签（例如 `ghcr.io/itsusinn/tuic-server:1.2.3`）。
- 镜像入口为 `tuic-server`，默认命令为 `-d /etc/tuic`，因此会读取挂载到 `/etc/tuic` 的配置目录。
- 容器内设置了 `IN_DOCKER=true`。

## 准备配置

容器不包含配置文件，需要先在宿主机上生成。可以借助一次性容器调用 `--init`：

```console
mkdir -p /etc/tuic
docker run --rm -v /etc/tuic:/etc/tuic ghcr.io/itsusinn/tuic-server --init
```

该命令会在 `/etc/tuic` 中生成一个包含 5 个随机用户的 `config.toml`；若文件已存在则不会覆盖。你也可以从[快速入门](getting-started.md)中的最小配置开始，或使用[配置生成器](/config-generator/)在浏览器本地生成成对的服务器与客户端配置。

请确认 `config.toml` 中的 `server` 监听地址允许来自容器的连接，例如 `[::]:8443`。

## 运行容器

### docker run

```console
docker run -d \
  --name tuic-server \
  --restart unless-stopped \
  -p 8443:8443/udp \
  -v /etc/tuic:/etc/tuic:ro \
  ghcr.io/itsusinn/tuic-server:latest
```

- `-p 8443:8443/udp` 必须显式指定 `/udp`；TUIC 不使用 TCP 监听端口。
- `-v /etc/tuic:/etc/tuic:ro` 以只读方式挂载配置目录，与默认命令 `-d /etc/tuic` 对应。
- 如需使用不同的目录，请覆盖命令，例如 `ghcr.io/itsusinn/tuic-server -d /config`。

### Docker Compose

```yaml
services:
  tuic-server:
    image: ghcr.io/itsusinn/tuic-server:latest
    container_name: tuic-server
    restart: unless-stopped
    ports:
      - "8443:8443/udp"
    volumes:
      - /etc/tuic:/etc/tuic:ro
```

若服务器使用 ACME 自动申请证书，HTTP-01 验证还需要开放 **TCP 端口 80**。此时可在 `ports` 中追加 `"80:80/tcp"`，或改用 `network_mode: host` 让容器直接使用宿主机网络。

## 验证

```console
docker logs tuic-server
```

日志应显示服务器正在监听并已准备好进行 TLS 连接。随后从本地机器向服务器发送一个 SOCKS5 请求进行验证。

## 排错

- **客户端无法连接**：确认 UDP 端口已放行、端口映射包含 `/udp`，以及云服务商的安全组允许 UDP。
- **TLS 验证失败**：确认客户端 `tls.sni` 与证书域名一致；自签名证书仅在测试时使用 `skip_cert_verify = true`。
- **ACME 申请失败**：确认 `80/tcp` 可达，且域名已正确解析到服务器。

通过配置验证仅表示字段有效；并不意味着 DNS、证书、防火墙或实际网络连接已通过验证。
