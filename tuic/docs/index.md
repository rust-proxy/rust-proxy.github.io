---
hide:
  - toc
  - navigation
---

# TUIC

通过 QUIC 连接应用与远端网络。

TUIC 为 TCP 和 UDP 流量提供代理通道。本手册介绍 **Itsusinn/tuic** 的独立服务端与客户端：在服务器上运行 `tuic-server`，在本机运行 `tuic-client`，再让应用使用本地 SOCKS5 代理。

## TUIC 如何工作

```text
应用 → 本地 SOCKS5 / 转发端口 → tuic-client
                                      │
                                  QUIC / UDP
                                      │
                                 tuic-server → 目标服务
```

应用到客户端是本地连接，客户端到服务端使用 QUIC。即使代理的是 TCP 请求，服务器也需要开放 **UDP** 监听端口。

[快速开始](getting-started.md){ .md-button .md-button--primary }
[生成配置文件](/config-generator/){ .md-button }

## 手册导航

- [快速开始](getting-started.md)：获取程序、最小配置、启动与验证。
- [服务端配置](server/config.md)：全部字段、TLS、ACL 与路由、出站、管理 API。
- [客户端配置](client/config.md)：连接字段、TLS、传输参数与端口转发。
- [配置生成器](tools/config-generator-reference.md)：在浏览器本地生成配对配置。
- [配置描述 DSL](tools/config-dsl.md)：生成器的 XML 描述格式。

## 其他链接

[程序下载](https://github.com/Itsusinn/tuic/releases) · [项目源码](https://github.com/Itsusinn/tuic) · [协议规范](/wind/tuic/)

!!! warning "开发版本"
    当前版本为 `2.0.0-dev7`，配置格式和功能仍可能调整。升级前请阅读 Release Notes 并备份配置。
