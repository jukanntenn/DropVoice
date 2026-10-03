# DV-RFC: Production TLS termination on the shared host Caddy

Status: proposed

## Problem

生产必须跑在 ttyo（alice @ 43.133.160.29）——一台共享 VPS，已有多个服务经同一个运维管理的 Caddy 挂在 :443 上，Cloudflare 对所有 zone 都走标准 443 回源（运维的统一标准，为 CDN 缓存行为保留）。仓库现有的生产设计——容器内 Caddy 以 Cloudflare Origin CA 证书在 :4443 终结 TLS，外加每 zone 一条 Origin Rule 端口改写——不适配这台机器：它会在共享监听器旁边再开一个 TLS 端口，多维护一个非标准端口和一条 zone 级规则（4443 不是 Cloudflare 原生可代理端口），且与机器上其他所有服务的接入方式不一致。

## Proposal

生产 TLS 终结于宿主共享 Caddy（markpost 模式）：Cloudflare → ttyo:443 → 宿主 Caddy（Origin CA 证书 SAN `dropvoice.online` + `*.dropvoice.online`，`trusted_proxies` = Cloudflare 段，`header_up X-Forwarded-For {http.request.header.CF-Connecting-IP}`）→ 反代到**仅回环**发布的容器 `127.0.0.1:8089` → 容器内 Caddy 保持 HTTP 三主机路由（`:8080`）→ axum。Caddyfile 模板删除 TLS 分支——所有环境同一份 HTTP 形态（staging：隧道终结；production：宿主 Caddy 终结）；证书 bind-mount 与 `tls_profile` 旋钮一并删除；生产的受信代理段变为 docker 网桥网关。宿主 Caddy 站点块是运维侧配置，记录在生产 runbook，不由本 playbook 管理。本记录只取代 [2026-08-30 拓扑记录](../implemented/2026-08-30-deterministic-three-host-public-topology.md) 的终结点部分——其确定性三主机布局、PWA/API 同源、真实 IP 纪律均不变。

## Alternatives considered

**保留已入库的设计（容器内 TLS :4443 + Origin Rule 改写）。** 败于运维的统一标准：账户里其他所有 zone 都已是纯 443 回源；第二个 TLS 监听器加一条 zone 级端口规则是无收益的额外维护，且 4443 不是 Cloudflare 原生可代理端口。

**markpost 的原样形态（容器内 TLS 走 :2053，原生可代理端口）。** 它省掉 Origin Rule，但仍在已有一台共享终结器的机器上按服务复制 TLS 与证书管理——多一个对外 TLS 端口、多一份证书生命周期，而且终究不是 443。

**让 DropVoice 容器直接占宿主 :443。** 平凡落选：443 已被服务其他服务的共享 Caddy 占有，容器拿不到。

## Acceptance criteria

- 部署后经公网 URL 的三主机 smoke 全绿（health + 版本、landing、PWA + service worker、注册往返）。
- 访问日志显示真实客户端 IP——Cloudflare → 宿主 Caddy → 容器 Caddy → axum 的 XFF 链穿越多出的一跳仍然成立；限流按客户端而非网关计键。
- 容器发布端口从机外不可达（仅回环绑定）。
- staging 与生产渲染出完全相同的 Caddyfile 形态（HTTP）；certs 挂载与 `tls_profile` 不复存在。

## Risks

- 真实 IP 链多出一跳且归运维侧配置所有：宿主 Caddy 站点块若丢了 `trusted_proxies` 或 CF-Connecting-IP 改写，所有用户共享一个限流桶（全部请求表现为网关 IP）——runbook 已把这两行标为不可省略。
- 容器的 `trusted_proxies_cidrs` 在首次部署收窄到实际网关 /32 之前，信任整个 docker 网桥段（group_vars 里的 TODO）。
- 宿主 Caddy 配置在本仓库 ansible 之外；runbook 片段可能与机器现实漂移——首次部署必须经公网 URL 验证（curl）。
- 回环端口 8089 是选定的默认值；ttyo 上若冲突，需同时改 group_vars 与宿主站点块。
