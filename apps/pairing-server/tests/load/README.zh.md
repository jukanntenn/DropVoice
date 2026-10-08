# 压力测试（spec 11 §13.3）

[English](README.md) | 中文

k6 压测脚本，验证配对服务器容量目标（spec 11 §10.4）：
100 万设备、峰值 1014 QPS。

## 运行

```bash
# 1. 启动本地同构测试环境（Caddy + axum，HTTP）
docker compose -f docker/docker-compose.local.yml up -d

# 2. 运行注册洪流压测
k6 run tests/load/register.js \
  -e BASE_URL=http://localhost:8080

# 3. 调大负载（模拟峰值 1014 QPS）
k6 run tests/load/register.js \
  -e BASE_URL=http://localhost:8080 \
  --vus 200 --duration 2m
```

## 脚本

| 脚本 | 场景 | 说明 |
|------|------|------|
| `register.js` | 注册洪流 | `POST /devices` 幂等 upsert，验证 201/200 区分与吞吐 |

后续可扩展：`heartbeat.js`（PUT /status 稳态）、`pairing_code.js`（读写混合）、
`sqlite_contention.js`（并发写入锁竞争）。

## 阈值（spec 11 §13.3）

- P95 延迟 < 100 ms
- 错误率 < 1%
- CPU < 80%（需结合 docker stats 观察）

## 合并门槛

配对服务器 PR 必须附带压测脚本 + 容量验证报告（spec 11 §13.3）。
