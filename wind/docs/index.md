---
hide:
  - toc
  - navigation
---

# Wind

Wind 是共享的代理框架与协议实现集合，为 TUIC、AnyTLS、NaïveProxy、Hysteria 2 等实现提供共用的传输、路由与配置基础设施。

本手册收录 Wind 的中文协议规范与设计文档。

## 协议规范

- [TUIC](tuic.md)：基于 QUIC 的代理协议。
- [AnyTLS](anytls.md)：在 TLS 之上多路复用的代理协议。
- [NaïveProxy](naive.md)：基于 HTTP/2 与 TLS 的代理协议。
- [Hysteria 2](hyteria.md)：基于 QUIC 的代理协议。

## 配置与路由

- [Config DSL](config-dsl.md)：静态配置描述语言。
- [ACL 中间表示](acl-ir.md)：nftables 形状的路由中间表示。

[源码仓库](https://github.com/rust-proxy/wind) · [文档仓库](https://github.com/rust-proxy/rust-proxy.github.io)
