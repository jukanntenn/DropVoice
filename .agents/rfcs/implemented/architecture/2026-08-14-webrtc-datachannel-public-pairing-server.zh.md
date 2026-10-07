# RFC: WebRTC DataChannel P2P with a public pairing-server rendezvous

Status: implemented

[English](2026-08-14-webrtc-datachannel-public-pairing-server.md) | 中文

## Problem

手机要把语音转文字送到 PC，PC 要接受来自手机的入站连接。最初的传输方案——桌面本地 axum HTTP/WS 服务器加 UDP 发现——要求两台设备处于同一 LAN 网段且防火墙宽松；桌面本地服务器的生命周期一旦与托盘隐藏、系统休眠纠缠就断连；手机在蜂窝网络下更是完全够不到 NAT 后面的桌面。配对还必须是扫二维码，而不是手输 IP，且逐键流动的文本绝不该存在第三方服务器上。

## Decision

文本经 **WebRTC DataChannel 点对点**流动：手机是 offerer，桌面是 answerer。任何一方的数据都不经过服务器；桌面完全不绑定本地数据端口。

**公网 pairing-server**（axum + SQLite，部署在 `ps.<域名>`）是纯粹的中继站（rendezvous）——只转发信令，绝不碰数据：

1. 桌面心跳注册设备（凭据受控的幂等 upsert；已存在设备重注册必须携带当前 Bearer token）并获得 pairing token。设备 token 只以 SHA-256 哈希落库、绝不复述；桌面收到 401 即重置设备身份（需重新扫码）。
2. 手机扫二维码（`dropvoice://pair?code=&device=&name=`），向 pairing-server POST WebRTC offer。
3. pairing-server 把 offer 中继给桌面的 SSE 订阅。桌面先用长效 Bearer token 换一张 60 秒一次性票据再打开事件流，长效 token 因此永不进入 URL 与访问日志。桌面的 **Rust 信令监督任务**消费 offer：用本地配对码/token 校验凭据，由 webrtc-rs 应答器（每个 offer 一个 `RTCPeerConnection`）生成 answer 并回填。SSE 活性靠 reqwest chunk 级 read timeout 确定性检出，半开连接的重建完全在 Rust 时钟内（休眠唤醒后 tokio 定时器立即触发）。
4. 手机长轮询 answer；收到 `accepted` 后 setRemoteDescription，DataChannel 打开。文本以 `{type:"text"}` 消息到达，桌面入队交给 Enigo 键盘注入。桌面在通道上发放连接 `token`，手机此后凭它重连、无需重新配对。

桌面的两条长连腿（心跳注册 + SSE 订阅）都在 Rust 进程内，并经同一配置源解析服务器 URL（`config.toml` 的 `network.pairing_server_url`，env 仅作开发编排覆盖）。webview 是纯视图、绝不参与信令——WebView2 会冻结隐藏页面的 JS 定时器与网络，托盘隐藏加休眠唤醒足以杀死页面侧信令。

## Alternatives considered

**保留本地 HTTP/WS 服务器 + UDP 发现。** 当时的在位方案。落选：同二层网段与防火墙要求挡住漫游中的手机和 NAT；本地服务器生命周期与窗口/托盘状态耦合；没有公网会合点，"蜂窝网络上的手机"到"家里的桌面"根本无路可走。

**文本经云端服务器中继。** 简单，穿透一切 NAT。落选：每次击键都要经过并驻留在第三方（对输入工具不可接受）；服务器带宽随使用量而非配对数扩展；信令解决之后直连 DataChannel 的延迟毫无劣势。

**用通用消息 broker（MQTT 或托管 realtime 服务）做信令。** 落选：会合点反正需要设备注册、token 发放与校验、限流、健康/冒烟界面——专用 axum 服务器一处同时提供注册与信令，不新增外部依赖。

**信令腿用 WebSocket 而非 SSE。** 落选：HTTPS 上的 SSE 原样复用心跳腿的 HTTP 客户端与 Bearer 认证，响应流的 chunk 级 read timeout 提供确定性的半开检出；ws 客户端会新增依赖和自己的保活机制，却换不来任何能力。

## Consequences

- pairing-server 保持纯会合点：只中继 offer/answer 与设备注册态，别无他物；其负载与打字量无关。数据面演进（如二进制帧）永远碰不到它。
- 桌面不绑定本地数据端口；托盘隐藏与休眠唤醒不再中断连接——两条长连腿（心跳、SSE）都在 Rust 进程内、靠 tokio 定时器重建。
- 长效凭据不进 URL 与日志（一次性 SSE 票据）；token 落库只存哈希；token 复用尝试得到 401 并重置桌面身份。
- 会合点的公网面按构造有界：30 天未上线的设备被 GC；进程内限流器对自身键数封顶（溢出重置而非膨胀）；豁免限流的端点（SSE、answer 长轮询）受全局并发帽保护——超限 429，绝不无界占用资源。
- 手机保持同源 PWA（开发：Vite 代理；部署：app 主机反向代理——主机布局另有专档，[2026-08-30](2026-08-30-deterministic-three-host-public-topology.zh.md)）——不存在构建期 API base，实际流量走 DataChannel。
- 未来任何把数据挪出 P2P 通道的传输变更必须取代本记录，而不是修改它。
