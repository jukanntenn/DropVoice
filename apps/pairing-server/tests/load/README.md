# Load testing (spec 11 §13.3)

English | [中文](README.zh.md)

k6 load-test scripts validating the pairing server's capacity targets (spec 11 §10.4):
1,000,000 devices, 1014 QPS peak.

## Running

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

## Scripts

| 脚本 | 场景 | 说明 |
|------|------|------|
| `register.js` | 注册洪流 | `POST /devices` 幂等 upsert，验证 201/200 区分与吞吐 |

后续可扩展：`heartbeat.js`（PUT /status 稳态）、`pairing_code.js`（读写混合）、
`sqlite_contention.js`（并发写入锁竞争）。

## Thresholds (spec 11 §13.3)

- P95 延迟 < 100 ms
- 错误率 < 1%
- CPU < 80%（需结合 docker stats 观察）

## Merge gate

配对服务器 PR 必须附带压测脚本 + 容量验证报告（spec 11 §13.3）。
