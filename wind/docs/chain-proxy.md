# 入站固定出口与链式代理

Wind 的入站可以通过 `outbound` 直接选择一个已声明的出口。显式绑定会跳过路由选择；引用不存在的出口时配置报错，不会回退到默认出口。省略该字段时保留普通路由行为，默认路由选择第一个声明的出口。

## Tunnel 入站

`type: tunnel` 将本地端口转发到固定目标。`tag`、`listen_addr`、`remote_addr` 和 `network` 必填，`network` 为 `tcp` 或 `udp`。UDP 的 `timeout` 是每个来源的空闲超时，默认为 `60s`，最小为 `4ns`。监听端口可以为 `0`，由系统分配；独立 Wind 配置中的其他出口需要填写实际端口，因此手工组装链路时使用固定监听端口。

`remote_addr` 接受域名、IPv4 或带方括号的 IPv6 地址及端口。域名不会由 Tunnel 自行解析，而是交给所选出口处理。

## SOCKS 出站

`type: socks` 使用 `tag` 和 `server_addr` 指定出口名称和 SOCKS5 服务器。`username`、`password` 可同时省略，或同时设置为各 1–255 字节的字符串。TCP 使用 CONNECT；UDP 需要显式设置 `allow_udp: true`，并要求服务器支持 UDP ASSOCIATE。

可选 `udp` 配置的默认值如下：

```yaml
udp:
  timeout: 8s
  buffer_size: 2048
  reconnect: false
  initial_backoff: 500ms
  max_backoff: 30s
```

`timeout` 限制建立 SOCKS5 UDP 会话的时间，必须为正。开启重连时，两个退避时间也必须为正。控制连接中断后，重试间隔逐次加倍，上限为 `max_backoff`；重连期间的数据报可能丢失。`buffer_size` 仅作为有界分配提示，不限制完整报文接收。实现不支持 SOCKS5 UDP 分片，会丢弃分片及非法报文。

UDP 请求中的目标域名交给 SOCKS 服务器解析，响应可以携带该服务器解析出的 IP 地址。仅接受协商所得 UDP 中继发来的响应，失败时不回退直连。

## SOCKS 上游承载 TUIC

下面的 YAML 展示独立 Wind 的组装方式。地址和凭据均为占位内容，使用前须替换；全零 UUID 不代表有效账户。

```yaml
inbounds:
  - type: socks
    tag: local
    listen_addr: 127.0.0.1:1080
    outbound: tuic
  - type: tunnel
    tag: __GENERATED_CHAINPROXY__tuic
    listen_addr: 127.0.0.1:10900
    remote_addr: tuic.example:124
    network: udp
    outbound: SOCKS_UPSTREAM

outbounds:
  - type: socks
    tag: SOCKS_UPSTREAM
    server_addr: socks.example:123
    allow_udp: true
    udp:
      reconnect: true
  - type: tuic
    tag: tuic
    server_addr: 127.0.0.1:10900
    sni: tuic.example
    uuid: 00000000-0000-0000-0000-000000000000
    password: "<TUIC password>"
```

本地 SOCKS 入站明确选择 `tuic`，避免因上游 SOCKS 排在出口列表首位而选错默认出口。Wind 先创建 SOCKS 出站并启动固定指向它的 Tunnel，再建立 TUIC 连接，随后开放普通业务入站。TLS 校验仍针对 `tuic.example`，而不是回环地址。

[tuic-client 的上游代理配置](/tuic/client/#socks5-upstream)会自动生成此拓扑，支持 Quinn 和 Quiche。它持续持有系统分配的本地端口，并将内部 Tunnel 限制为预先绑定的 TUIC 本地发送 socket；普通 Tunnel 则可服务多个来源。
