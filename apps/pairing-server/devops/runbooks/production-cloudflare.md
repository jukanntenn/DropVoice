# 生产 Cloudflare + R2 一次性配置 runbook

上线 `dropvoice.online` 前的一次性面板与宿主机配置清单（zone 迁移、回源、更新分发）。部署命令与发布流程在 [docs/development.md](../../../../docs/development.md)，本文只管面板与宿主 Caddy 操作；全部完成后无需重做。

## 0. 前置事实

- 生产源站：ttyo（alice @ 43.133.160.29；inventory 见 `hosts.yml` + `host_vars/ttyo.yml`）
- 流量链（markpost 模式，与该机既有服务一致）：Cloudflare 回源 `ttyo:443` → 宿主共享 Caddy 终结 TLS（Origin CA 证书）→ 按主机名反代容器**仅回环**发布的 `127.0.0.1:8089` → 容器内 Caddy（HTTP `:8080`）三站点分流 → axum（`127.0.0.1:38424`）
- 部署目录（alice）：`/home/alice/docker/dropvoice/`（data/ + log/；容器恒 HTTP，无 certs 目录）
- 无 Origin Rule 端口改写、无容器内证书：回源就是标准 443，与其他服务零差别

## 1. zone 迁移（DNS → Cloudflare）

- [ ] 注册商处把 `dropvoice.online` 的 NS 改为 Cloudflare 分配的两条 NS，等待 zone Active
- [ ] zone 的 SSL/TLS 加密模式设为 Full (strict)（仅作用于本 zone）

## 2. 三主机 DNS + 宿主共享 Caddy

- [ ] DNS 添加三条 A 记录（全部橙云代理）：`dropvoice.online`、`app`、`ps` → `43.133.160.29`（回源 443，无需任何 Origin Rule）
- [ ] SSL/TLS → Origin Server → Create Certificate：SAN 覆盖 `dropvoice.online` + `*.dropvoice.online`（默认 15 年），得到 `origin.pem` / `origin.key`
- [ ] 证书放入 ttyo 宿主 Caddy 的证书目录（沿用该机既有布局，如 `/etc/caddy/certs/`，路径以机器现状为准）
- [ ] 宿主 Caddyfile 追加三站点块（可直接粘贴；证书路径按上一步调整）：

```
dropvoice.online, app.dropvoice.online, ps.dropvoice.online {
	tls /etc/caddy/certs/dropvoice-origin.pem /etc/caddy/certs/dropvoice-origin.key
	reverse_proxy 127.0.0.1:8089 {
		trusted_proxies 173.245.48.0/20 103.21.244.0/22 103.22.200.0/22 103.31.4.0/22 141.101.64.0/18 108.162.192.0/18 190.93.240.0/20 188.114.96.0/20 197.234.240.0/22 198.41.128.0/17 162.158.0.0/15 104.16.0.0/13 104.24.0.0/14 172.64.0.0/13 131.0.72.0/22
		header_up X-Forwarded-For {http.request.header.CF-Connecting-IP}
	}
}
```

> 注：宿主 Caddy 较旧，`trusted_proxies` 用裸 CIDR 列表（同 markpost 在该机的
> 写法），不要加 `static` 限定词（新版语法，旧版报
> `invalid IP address: 'static'`）。

- [ ] 重载宿主 Caddy（`systemctl reload caddy` 或该机等价命令）
- [ ] 回环端口：`127.0.0.1:8089` 如与 ttyo 既有服务冲突，换任意空闲回环端口，同步改 `group_vars/production.yml` 的 `host_port` 与上面站点块
- [ ] （暂缓）限流 / WAF / Cache 规则：应用内已有每 IP 1 rps + 30 天设备 GC，试运营后补

> 真实 IP 链（不可省略的两行）：宿主 Caddy 信任 Cloudflare 段并把 XFF 改写为
> `CF-Connecting-IP` → 容器 Caddy 信任 docker 网桥网关 → axum 取最左 XFF。站点块
> 缺了 `trusted_proxies`/`header_up` 两行时，所有用户会被算成同一个 IP——限流
> 直接失效（全站共享一个桶）。

## 3. R2 更新分发（releases.dropvoice.online）

- [ ] 账户绑定支付方式（R2 开通硬前提；试运营用量在免费额度内为 $0：每月 10 GB 存储 / 100 万 Class A / 1000 万 Class B，出口流量免费）
- [ ] Storage & Databases → R2 → 完成订阅开通（$0 checkout）
- [ ] 创建 bucket：`dropvoice-releases`
- [ ] bucket → Settings → Custom Domain → 绑定 `releases.dropvoice.online`（DNS 记录与边缘证书自动生成）
- [ ] 验证空态：`curl -sI https://releases.dropvoice.online/update/manifest.json` 返回 404（尚无对象，属正常）

## 4. 密钥与 repo secrets

- [ ] API Token：My Profile → API Tokens → Create Custom Token，权限仅 `Account → R2 → Edit`
      `gh secret set CLOUDFLARE_API_TOKEN`（粘贴 token 值）
- [ ] 更新签名私钥（本机 `~/.tauri/dropvoice-updater.key`，空密码；对应公钥已烘焙进 tauri.conf.json）：
      `gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/dropvoice-updater.key`
      （`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 可省略——密钥无密码）
      ⚠️ 私钥一旦丢失，已装客户端永远无法再收到更新——请立即把文件备份进密码管理器
- [ ] 镜像：CI 发布到 ghcr.io（内建 GITHUB_TOKEN，无需 registry secret）；**首次发布后**
      在 GitHub → Packages → `dropvoice-pairing-server` → Package settings 把可见性改为
      **Public**——生产 VPS 匿名拉取依赖它
- [ ] Apple / Windows 代码签名：试运营阶段不启用（产物未签名：macOS 需右键打开绕过 Gatekeeper，Windows 有 SmartScreen 警告）。启用是付费决策（Apple Developer Program / 代码签名证书），且 Windows 侧还需补 workflow 证书导入步骤

## 5. 完成后

- 回到 [docs/development.md](../../../../docs/development.md) 走发布流程；首个稳定版 tag 推送后 `release.yml` 的 `publish-r2` job 应为绿，且 `https://releases.dropvoice.online/update/manifest.json` 返回新版本号
- R2 控制台应出现 `files/vX.Y.Z/` 与 `update/manifest.json`
- 生产部署后全链验证：`python scripts/smoke.py https://dropvoice.online --app-url https://app.dropvoice.online --api-url https://ps.dropvoice.online --expect-version vX.Y.Z`（5/5 PASS）
