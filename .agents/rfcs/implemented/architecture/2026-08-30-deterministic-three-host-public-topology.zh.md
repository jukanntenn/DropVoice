# RFC: Deterministic three-host public topology (landing / app / ps)

Status: implemented

[English](2026-08-30-deterministic-three-host-public-topology.md) | 中文

## Problem

一台 Cloudflare 之后的小 VPS 要公开服务三种内容——营销 landing、可安装的手机 PWA、纯 JSON 信令 API。PWA 访问 API 必须完全不需要 CORS 机制；桌面的 Rust HTTP 腿需要稳定的 API 主机；限流必须穿过 CDN 找回真实客户端 IP；发布不能让前端与后端版本漂移；而 staging dogfooding 环境必须原样演练生产形态——否则它什么都验证不了。

## Decision

把公网面拆成**三个确定性主机，按 Host 头分流——一个 URL 恒定一个内容，无 UA 嗅探**：

- `<domain>`（apex）——landing，来自 `landing-dist`
- `app.<domain>`——手机 PWA（`mobile-dist`），`/api` 同源反代
- `ps.<domain>`——纯 API，服务桌面的 Rust 腿（heartbeat + SSE）

生产是 Cloudflare 之后的 `dropvoice.online`；staging 一一映射（`*.dropvoice.bytehome.fun`），翻转三组值——域名、TLS 形态、受信代理段——即可原样搬进 Cloudflare。单一模板 `devops/ansible/templates/Caddyfile.j2` 同时渲染两个环境（staging HTTP、隧道终结 TLS；production TLS、Cloudflare Origin CA 证书）。

两个前端（手机 PWA + landing）都打进 pairing-server 镜像：镜像是唯一自包含的发布产物，前端版本不可能与后端漂移。

真实客户端 IP 链是 server 级 `trusted_proxies`（上游代理段）+ `trusted_proxies_strict`（从右往左信任链解析）+ `header_up X-Forwarded-For {client_ip}`：未受信的直连对端会被改写掉伪造的 XFF，axum 的最左 XFF 解析因此恒得真实客户端——限流键与访问日志按构造一致。

"PWA 必须同源"本身是 [2026-08-14 WebRTC DataChannel 记录](2026-08-14-webrtc-datachannel-public-pairing-server.zh.md)的后果（"手机保持同源 PWA"）；本记录决定满足它的主机布局。机制事实——哪个文件渲染什么、端口、缓存头——住在 [docs/architecture.zh.md](../../../../docs/architecture.zh.md) 与 [docs/development.zh.md](../../../../docs/development.zh.md)。

## Alternatives considered

**在位方案：单个公网 origin（`api.dropvoice.app`）在同一主机上服务 PWA + API。** 落选：消费级 PWA 挂在一个写着 "api" 的主机名下，同一产物上也没有 landing 的容身之处；且每个环境都要推导并维护 CORS origin 列表（公网 URL + tauri webview origin），而信令迁入 Rust 进程之后，跨域调用者早已不存在。

**单主机 + UA 嗅探，按设备挑 PWA / API / landing 内容。** 落选：路由必须确定性——一个 URL 恒定一个内容。依赖 UA 的响应会毒化边缘缓存（Cloudflare 按 URL 缓存）、让排障不可复现，还在每台手机面前放了一个"发错变体"的静默失败模式。

**landing 放独立静态主机，镜像只带 PWA。** 落选：landing 广告 PWA 入口 URL，独立部署的 landing 可能与后端对"手机该去哪"产生分歧；且重新引入按发布人工上传产物——这正是 PWA 打进镜像已经消灭的过期类别。

**保留两份 Caddyfile 产物——staging 用 HTTP 拷贝、production 专用 TLS 模板。** 落选：生产路由住在 staging 从不渲染的文件里，staging 验证的是与生产不同的 Caddyfile——这正是 staging 存在要抓的失效。两份渲染还要求每次路由变更手工镜像同步。

## Consequences

- `cors_origins` 在所有标准环境（dev 任务、验收、staging、production）均为空；仅作为特殊实验的逃生舱保留。桌面的默认服务器 URL 是 `https://ps.dropvoice.online`（`network.pairing_server_url`）。
- HTML 永不进边缘缓存（`no-cache`），哈希资产 `immutable`——按 URL 的确定性正是 3 Mbps 源站敢用 Cloudflare 边缘缓存的前提。
- Cloudflare 面板前置（三条橙云记录、Full strict、Origin Rule 端口改写、限流/WAF/缓存规则）属于生产 runbook，不在仓库里。
- staging 演练的就是生产的 Caddyfile 模板本身，只有值不同；部署后的冒烟检查探测全部三个主机（`--app-url` / `--api-url`）。
- 一次发布发一个镜像，同时携带后端 + PWA + landing；landing 的 `pwaUrl` 与后端不可能再各执一词。
