---
hide:
  - toc
  - navigation
---

# TUIC

TUIC 是一个基于 QUIC 传输协议实现的 0-RTT 代理协议。旨在最大限度地降低握手延迟，同时提供安全可靠的代理功能，并支持TCP和UDP流量。它充分利用了QUIC的优势，包括连接迁移、多路复用能力以及缩短的握手时间。

有关协议规范的具体细节，请参阅[《TUIC 协议》](https://rust-proxy.github.io/wind/tuic/)；


## 手册导航

- [快速开始](getting-started.md)：获取程序、最小配置、启动与验证。
- [服务端](server.md)：安装、启动与完整配置、TLS 与 ACME。
- [客户端](client.md)：安装、启动与完整配置、SOCKS5 与端口转发。
- [Docker 部署](docker.md)：使用官方镜像运行服务端。
- [配置编辑器](/config-editor/)：在浏览器本地分别生成服务端与客户端配置，预览区逐行显示字段说明。

## 其他链接

[程序下载](https://github.com/Itsusinn/tuic/releases) · [项目源码](https://github.com/Itsusinn/tuic) · [协议规范](/wind/tuic/)

