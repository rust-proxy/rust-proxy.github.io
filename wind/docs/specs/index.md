---
hide:
  - toc
---

# Wind Protocol Specifications

Wind is a shared proxy framework and a collection of protocol implementations. It provides common transport, routing, and configuration infrastructure for TUIC, AnyTLS, NaïveProxy, Hysteria 2, and other implementations.

This section contains the English editions of the Wind protocol specifications and design documents.

## Protocol Specifications

- [TUIC](tuic.md): A QUIC-based proxy protocol.
- [AnyTLS](anytls.md): A multiplexed proxy protocol over TLS.
- [NaïveProxy](naive.md): A proxy protocol based on HTTP/2 and TLS.
- [Hysteria 2](hyteria.md): A QUIC-based proxy protocol.

## Configuration and Routing

- [Config DSL](config-dsl.md): A static configuration description language.
- [ACL Intermediate Representation](acl-ir.md): An nftables-shaped routing intermediate representation.

## Template

- [RFC Template](example.md): A template for drafting new specifications.

[简体中文](../index.md) · [Source repository](https://github.com/rust-proxy/wind) · [Documentation repository](https://github.com/rust-proxy/rust-proxy.github.io)
