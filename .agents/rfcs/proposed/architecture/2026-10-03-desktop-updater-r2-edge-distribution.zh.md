# RFC: Desktop updater distribution via Cloudflare R2 edge

Status: proposed

[English](2026-10-03-desktop-updater-r2-edge-distribution.md) | 中文

## Problem

每一个发出的桌面安装包都把 Tauri updater 端点烘焙进二进制（`apps/desktop/src-tauri/tauri.conf.json` 的 `plugins.updater`）；安装到用户手里之后，端点只能靠"经由它发一次更新"来更换。仓库里烘焙的端点（`releases.dropvoice.app`）指向已实现的公开拓扑（[2026-08-30](../../implemented/architecture/2026-08-30-deterministic-three-host-public-topology.zh.md)）从未定义的域名——没有宿主、没有 manifest 生成器；构建还缺 `createUpdaterArtifacts`，签名更新制品根本不存在：更新链路断了三处。在首次对外安装（试运营）之前必须定下更新从哪里服务：全球可达（zh 是一等 locale）、不占用 3 Mbps 的源站 VPS、不扩大 ansible 管理的源站面。

## Proposal

更新分发由 Cloudflare R2 以 `releases.dropvoice.online` 承担——挂在既有 `dropvoice.online` zone 上的自定义域，边缘缓存、出口零费用。稳定版 tag 触发 `release.yml` 新增的 `publish-r2` job：下载 GitHub release 资产（tauri-action 的 `latest.json` 与已签名安装包），把各平台 URL 改写为 `<domain>/files/<tag>/<filename>`（`scripts/publish_updater_manifest.py`），经 wrangler 上传——制品 `immutable`、`update/manifest.json` `no-cache`。预发布 tag 经 CI 永不移动 manifest，与 `docker-publish.yml` 治理 docker `latest` 的稳定版规则同源；发布脚本上的手工 `--allow-prerelease` 逃生口仅用于在首个稳定版发布前（尚无装机量可惊扰时）彩排更新链路。`tauri.conf.json` 里无主的原公钥换成新生成的 minisign 密钥对（私钥在仓库外保管，以 `TAURI_SIGNING_PRIVATE_KEY` secret 交付）；装机量为零的当下轮换是免费的，首发之后即不可能。试运营量级的美元成本为零——R2 免费额度（每月 10 GB 存储、100 万 Class A / 1000 万 Class B、出口免费）对约 100 MB 的一次发布绰绰有余——但开通 R2 要求账户绑有支付方式。

## Alternatives considered

**用 GitHub Releases 当 updater 端点**（`…/releases/latest/download/latest.json`；零新基建，tauri-action 本来就上传 `latest.json`）。落选于可达性：github.com 的下载 URL 在中国大陆不可靠，而 zh 是一等 locale。它还把可用性与限流让渡给第三方，且没有我们自己的缓存头控制。

**在生产 VPS 上加第四个静态文件宿主。** 落选于：它把发布分发与信令源站耦合——一次坏部署或下载尖峰会伤到配对链路——并消耗三主机拓扑刻意保护的 3 Mbps 源站带宽，换来的只是 Cloudflare 边缘免费就能服务的纯静态字节。

**什么都不做（端点保持死链）。** 落选于：每次更新都得手动重装，而错误端点继续烘焙进每个发出的安装包——放弃了唯一能迁移已装客户端的机制，且恰恰在修复成本变得高昂的时刻（穿越死端点发更新是唯一的迁移路径）。

## Acceptance criteria

- release 构建产出 `.sig` 文件与 `latest.json`（`createUpdaterArtifacts` 开启、签名 secret 在位）。
- 稳定版 tag 之后，`https://releases.dropvoice.online/update/manifest.json` 返回新版本号且平台条目齐全；每个 `url` 可解析（200、`immutable`），每个 `signature` 是对应 `.sig` 的内容。
- 预发布 tag 经 CI 不触碰 manifest；`--allow-prerelease` 是预发布移动 manifest 的唯一路径，显式且高声警告。
- 已安装的旧版本弹出更新提示并经端点端到端完成升级。

## Risks

- R2 开通要求绑定支付方式，超出免费额度的用量按量计费（Class B 读取每百万 $0.36）——试运营量级下敞口有界且遥远，但不是硬封顶的零。
- 新 minisign 私钥是一次性基建：首发前丢失可自由轮换；首发后丢失，已装客户端永远无法再收到更新。创建它的同一个变更里就必须完成备份（密码管理器）。
- 流水线依赖 `CLOUDFLARE_API_TOKEN` secret（Account → R2 → Edit）；token 缺失会让 `publish-r2` 在 GitHub release 已存在之后响亮失败——恢复手段是重跑该 job 或手工执行脚本。
