# Staging 验收 runbook（dogfooding）

发布前先跑本清单；任何一步失败即停线，用 `deploy.py staging --tag <prev>` 回滚。

## 0. 前置

- [ ] 本机质量门绿：`pnpm quality`（= prek 全仓 format + lint + gates + tests）
- [ ] 镜像已推送：`pnpm image:push`（LAN registry，tag: main）
- [ ] 本地容器验收绿：`pnpm accept:up` → `pnpm accept:test`（e2e-caddy + smoke，
      配置为入库的 docker/config.acceptance.toml）→ `pnpm accept:down`

## 1. 部署

- [ ] `pnpm deploy:staging`
- [ ] 冒烟已随部署自动执行：playbook 末尾会跑 `scripts/smoke.py https://dropvoice.bytehome.fun
      --expect-version <部署 checkout 的 git describe>`（4/4 PASS，含版本比对）；
      部署命令退出码非 0 即失败。如需复核：`python scripts/smoke.py https://dropvoice.bytehome.fun`
- [ ] 运维确认：隧道仍指向 fn:8080（如切换过端口，与运维同步）

## 2. 真机 dogfooding（5-10 分钟）

- [ ] 桌面：`pnpm build:tauri` → 本机安装安装包；`<config_dir>/dropvoice/config.toml`
      设 `network.pairing_server_url = "https://ps.dropvoice.bytehome.fun"`
- [ ] 手机：浏览器打开 `https://dropvoice.bytehome.fun`（PWA 可安装）
- [ ] 配对：手机显示配对码 → 桌面确认配对成功（两台设备互见）
- [ ] 发送：手机真实语音输入 → 桌面键盘注入成功
- [ ] 反向：重启桌面应用 → 手机 token 重连成功（无需重新配对）
- [ ] 数据检查：staging 部署目录日志无 ERROR；`/health` 的 schema_version 与预期一致

## 3. 放行

- [ ] 全部通过 → 记录部署的镜像 tag（`git describe`），staging 保持运行供持续 dogfooding
