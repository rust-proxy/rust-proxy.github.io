# Docker 部署

本页面介绍如何使用官方镜像运行 `tuic-server`。服务端的完整配置见[服务端](server.md)，客户端仍按[快速入门](getting-started.md)在宿主机或其它容器中运行。

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
- `latest` 指向最近一次发布；也可以使用对应的版本标签（例如 `ghcr.io/itsusinn/tuic-server:1.2.3`）。镜像仅在推送 `v*` 版本标签时发布，普通分支或 PR 构建不会推送。
- 镜像入口为 `/usr/bin/tuic-server`，默认命令为 `-d /etc/tuic`，因此会读取挂载到 `/etc/tuic` 的配置目录。
- 容器的工作目录为 `/var/lib/tuic`；服务器按配置文件扩展名判断格式，未识别的扩展名需要显式设置 `TUIC_CONFIG_FORMAT`。
- 最终阶段基于 `gcr.io/distroless/cc`：包含 glibc 运行时与 CA 证书（可用于 ACME 申请），但不含 shell、包管理器及 `curl`/`wget` 等工具，因此无法 `docker exec` 进入容器调试。

## 准备配置

容器不包含配置文件，需要先在宿主机上生成。可以借助一次性容器调用 `--init`；该命令会把 `config.toml` 写入进程的当前工作目录，因此必须用 `-w` 指定到挂载目录：

```console
mkdir -p /etc/tuic
docker run --rm -w /etc/tuic -v /etc/tuic:/etc/tuic ghcr.io/itsusinn/tuic-server --init
```

该命令会在 `/etc/tuic` 中生成一个包含 5 个随机用户的 `config.toml`；若当前目录已存在 `config.toml`，命令会报错退出，不会覆盖已有文件。你也可以从[快速入门](getting-started.md)中的最小配置开始，或使用[配置生成器](/config-generator/)在浏览器本地生成成对的服务器与客户端配置。

请确认 `config.toml` 中的 `server` 监听地址允许来自容器的连接，例如 `[::]:8443`。

## 运行容器

### docker run

```console
docker run -d \
  --name tuic-server \
  --restart unless-stopped \
  -p 8443:8443/udp \
  -v /etc/tuic:/etc/tuic:ro \
  -v /var/lib/tuic:/var/lib/tuic \
  ghcr.io/itsusinn/tuic-server:latest
```

- `-p 8443:8443/udp` 必须显式指定 `/udp`；TUIC 不使用 TCP 监听端口。
- `-v /etc/tuic:/etc/tuic:ro` 以只读方式挂载配置目录，与默认命令 `-d /etc/tuic` 对应。
- `-v /var/lib/tuic:/var/lib/tuic` 以可写方式挂载数据目录，用于持久化 `data_dir`（ACME 证书等）；如不需要持久化可省略。
- 如需使用不同的目录，请覆盖命令，例如 `ghcr.io/itsusinn/tuic-server -d /config`。

容器的工作目录是 `/var/lib/tuic`。配置中的 `data_dir` 默认为空，会解析为该工作目录；TLS 证书与私钥的**相对路径**也基于 `data_dir` 解析，而不是配置文件所在目录。因此配置里应使用**绝对路径**，或显式设置 `data_dir`，否则相对路径会指向容器内临时的 `/var/lib/tuic`。如需持久化 ACME 等数据，请把宿主机目录挂载到该路径。

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
      - /var/lib/tuic:/var/lib/tuic
```

若服务器使用 ACME 自动申请证书，HTTP-01 验证还需要开放 **TCP 端口 80**。此时可在 `ports` 中追加 `"80:80/tcp"`，或改用 `network_mode: host` 让容器直接使用宿主机网络。

ACME 证书会写入 `data_dir`。默认情况下 `data_dir` 为空并解析为容器内 `/var/lib/tuic`，属于容器可写层，容器重建后会丢失并触发重新申请。如需持久化，把宿主机目录挂载到容器内的 `/var/lib/tuic` 即可，无需修改 `data_dir`，配置目录也仍可保持 `:ro`。挂载示例见上文的 `docker run` 与 Docker Compose。

## 验证

```console
docker logs tuic-server
```

日志应显示服务器正在监听并已准备好进行 TLS 连接。随后从本地机器向服务器发送一个 SOCKS5 请求进行验证。

## 排错

- **客户端无法连接**：确认 UDP 端口已放行、端口映射包含 `/udp`，以及云服务商的安全组允许 UDP。
- **TLS 验证失败**：确认客户端 `tls.sni` 与证书域名一致；自签名证书仅在测试时使用 `skip_cert_verify = true`。
- **ACME 申请失败**：确认 `80/tcp` 可达，且域名已正确解析到服务器。
- **无法进入容器排查**：镜像基于 Distroless，不含 shell，`docker exec ... sh` 不可用；请使用 `docker logs` 查看日志，并通过挂载的配置目录定位问题。
- **提示证书或数据文件不存在**：相对路径基于 `data_dir`（默认容器工作目录 `/var/lib/tuic`）解析，而非配置文件所在目录；请改用绝对路径或设置 `data_dir`。

通过配置验证仅表示字段有效；并不意味着 DNS、证书、防火墙或实际网络连接已通过验证。
