# Staging 验收 runbook（dogfooding）

发布前先跑本清单；任何一步失败即停线，用 `deploy.py staging --tag <prev>` 回滚。

## 0. 前置

- [ ] 本机质量门绿：`pnpm quality` + `pnpm test:rust`
- [ ] 镜像已推送：`python apps/pairing-server/docker/build.py --push`（LAN registry）
- [ ] 本地容器验收绿：`docker compose -f apps/pairing-server/docker/docker-compose.local.yml up -d --build`
      → `cargo test -p dropvoice-pairing-server --test e2e-caddy -- --ignored`
      → `python scripts/smoke.py http://localhost:8080`
      → `docker compose -f apps/pairing-server/docker/docker-compose.local.yml down -v`

## 1. 部署

- [ ] `python apps/pairing-server/devops/deploy.py staging`
- [ ] 部署后冒烟：`python scripts/smoke.py https://dropvoice.bytehome.fun`（4/4 PASS）
- [ ] 运维确认：隧道仍指向 fn:8080（如切换过端口，与运维同步）

## 2. 真机 dogfooding（5-10 分钟）

- [ ] 桌面：`pnpm build:tauri` → 本机安装安装包（信令默认指向 dropvoice.bytehome.fun）
- [ ] 手机：浏览器打开 `https://dropvoice.bytehome.fun`（PWA 可安装）
- [ ] 配对：手机显示配对码 → 桌面确认配对成功（两台设备互见）
- [ ] 发送：手机真实语音输入 → 桌面键盘注入成功
- [ ] 反向：重启桌面应用 → 手机 token 重连成功（无需重新配对）
- [ ] 数据检查：staging 部署目录日志无 ERROR；`/health` 的 schema_version 与预期一致

## 3. 放行

- [ ] 全部通过 → 记录部署的镜像 tag（`git describe`），staging 保持运行供持续 dogfooding
