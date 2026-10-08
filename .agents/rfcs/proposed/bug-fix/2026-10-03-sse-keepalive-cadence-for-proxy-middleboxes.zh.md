# RFC: SSE keepalive cadence for aggressive proxy middleboxes

Status: proposed

[English](2026-10-03-sse-keepalive-cadence-for-proxy-middleboxes.md) | 中文

## Problem

桌面端维持一条 SSE 长连接（`GET /api/devices/{id}/webrtc/events`）接收手机的 incoming offer。15s 心跳周期（spec §4.6，按 Cloudflare 125s Proxy Read Timeout 定的）隐含假设：客户端路径上的中间设备能容忍 ≥15s 的静默。生产环境推翻了这个假设：一台经办公室出口走商业代理链的桌面，其 events 流**在最后一个字节流过之后 ~5s 被 cancel**（容器 Caddy 记录 `aborting with incomplete response: reading: context canceled`；无事件时建连 +5s 于响应头被杀，有事件时 +5s 于最后一个 offer 事件——滚动 5s 空闲规则）。客户端自身读超时 45s、Cloudflare 下限 100s+、服务器无此上限，凶手只能是代理链。

后果：桌面订阅反复翻腾（subscribe → ~5s 空窗 → 重订），手机 offer 落在空窗内即被 503 拒绝（"应答方不在线"），offer 重试突发触发按 IP 限流（429），手机 UI 表现为"扫码连上后立刻断开"。大陆用户普遍在企业代理后面（zh 是一等语言），这是产品级缺陷，不是单一办公室的怪癖。

## Proposal

关闭空闲窗口，而不是假设它不存在：

- 心跳周期 15s → **3s**（`SSE_PING_INTERVAL`）：对实测 5s 规则留 2s 余量；每 ping ~45 B，每在线桌面 ~15 B/s。具名 `ping` 事件与 axum 的 `KeepAlive` 注释流统一从这一个常量取值。
- **建连瞬间发首个 ping**（t=0），不再静默消费 tokio interval 立即触发的第一个 tick：以"首个 body 字节"判定空闲的中间设备在流打开的瞬间就看到数据；此前流在建连后整整一个周期内只有响应头、零 body。

Cloudflare 125s 与 staging openresty 300s 上限依旧大幅满足；240s 优雅生命周期 + `retry:` 重连协议不变。

## Alternatives considered

**保持 15s，让受影响用户为我们的域名绕过代理。** 落选原因：用户网络不由我们控制——语音输入工具必须能在任何企业出口后面工作，试运营规模下"去修你的代理"不是可提供的支持答案。

**改用 WebSocket 替代 SSE。** 落选原因：改动面太大——信令协议、票据认证、重连语义与手机侧长轮询都围绕 HTTP SSE 设计；且滚动 5s 空闲规则对空闲 WebSocket 同样致命——问题不在传输层，在静默本身。

**每 1s 一次 ping。** 落选原因：成本纪律——相比 3s 只是翻倍安全余量而无实证收益，而每桌面的信令流量要过 Cloudflare、落在 3 Mbps 源站上，应停留在证据支持的最低水平。

## Acceptance criteria

- 生产代理链后的桌面能在 ping 边界上保持单条 events 流存活（服务器日志无重订翻腾；`aborting with incomplete response` 消失）。
- events 响应的首批字节在建连瞬间即离开服务器（`curl -N` 在任何周期间隔之前看到第一个 `ping`）。
- 流建立期间的手机配对不再出现由订阅空窗导致的 503 offer 拒绝。
- openapi 描述与内嵌心跳周期保持同步（同一提交内重新生成）。

## Risks

- 若存在比 3s 更严的空闲规则仍会杀流；节奏按实测证据（5s）调校，非保证——桌面端 1s 提示重连把症状限定为有界抖动而非死锁。
- ping 频率 ×5 = 信令字节 ×5；每桌面 ~15 B/s 在试运营规模可忽略，但连接量上涨数量级后需要重新审视这一项。
