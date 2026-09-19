---
hide:
  - toc
  - navigation
---

# TUIC

本文档全面介绍了 TUIC 项目，这是一个基于 QUIC 传输协议实现的 0-RTT 代理协议。本文档的重点是阐述 TUIC 系统的目的、架构、组件及关键特性。

TUIC旨在最大限度地降低握手延迟，同时提供安全可靠的代理功能，并支持TCP和UDP流量。它充分利用了QUIC的优势，包括连接迁移、多路复用能力以及缩短的握手时间。

TUIC 旨在最大限度地降低握手延迟，同时提供安全可靠的代理功能，并支持 TCP 和 UDP 流量。它充分利用了 QUIC 的优势，包括连接迁移、多路复用能力以及缩短的握手时间。

有关协议规范的具体细节，请参阅[《TUIC 协议》](https://rust-proxy.github.io/wind/tuic/)；


## 手册导航

- [快速开始](getting-started.md)：获取程序、最小配置、启动与验证。

## 其他链接

[程序下载](https://github.com/Itsusinn/tuic/releases) · [项目源码](https://github.com/Itsusinn/tuic) · [协议规范](/wind/tuic/)

