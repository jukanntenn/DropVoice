// k6 压测脚本：注册洪流 + 心跳稳态（spec 11 §13.3）。
//
// 运行：k6 run tests/load/register.js
// 阈值：P95 < 100ms、错误率 < 1%（spec 11 §13.3）。
//
// 目标：验证容量 100 万设备、峰值 1014 QPS（spec 11 §10.4）。
// 本脚本聚焦注册路径（POST /devices 幂等 upsert）。心跳/配对码可扩展更多脚本。

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter } from 'k6/metrics';

const BASE = __ENV.BASE_URL || 'https://localhost:4443';
const SCENE = __ENV.SCENE || 'register'; // register | heartbeat | mixed

// 自定义指标：201（新建）vs 200（复用）
const created201 = new Counter('devices_created_201');
const reused200 = new Counter('devices_reused_200');

export const options = {
  scenarios: {
    // 场景 1：注册洪流——大量新设备并发注册。
    register: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 200 }, // ramp up
        { duration: '1m', target: 200 }, // 稳态 200 并发
        { duration: '30s', target: 0 }, // ramp down
      ],
      gracefulRampDown: '10s',
    },
  },
  thresholds: {
    // spec 11 §13.3 阈值
    http_req_duration: ['p(95)<100'],
    http_req_failed: ['rate<0.01'],
  },
};

// 每个虚拟用户维护一个稳定的 device_id（首次新建，之后复用）。
const DEVICE_ID = `00000000-0000-4000-8000-${__VU.toString(16).padStart(12, '0')}`;

export default function () {
  const payload = JSON.stringify({
    device_id: DEVICE_ID,
    platform: 'desktop',
    device_name: `k6-vu-${__VU}`,
    address: { ip: `10.0.0.${(__VU % 254) + 1}`, port: 38425 },
  });

  const params = {
    headers: { 'Content-Type': 'application/json' },
    insecureSkipTLSVerify: true, // 本地自签证书（spec 11 §13.1）
  };

  const res = http.post(`${BASE}/devices`, payload, params);

  const ok = check(res, {
    'status is 201 or 200': (r) => r.status === 201 || r.status === 200,
    'has pairing_token': (r) => r.json('pairing_token') !== undefined,
  });

  if (ok) {
    if (res.status === 201) created201.add(1);
    if (res.status === 200) reused200.add(1);
  }

  sleep(0.1); // 每虚拟用户 ~10 req/s
}

export function handleSummary(data) {
  return {
    stdout: textSummary(data),
    'tests/load/register-report.json': JSON.stringify(data, null, 2),
  };
}

// 精简文本摘要（k6 内置 summary 较冗长）。
function textSummary(data) {
  const m = data.metrics;
  const reqs = (m.http_reqs && m.http_reqs.values.count) || 0;
  const dur = (m.http_req_duration && m.http_req_duration.values) || {};
  const fail = (m.http_req_failed && m.http_req_failed.values.rate) || 0;
  return (
    `\n=== k6 压测报告 (${SCENE}) ===\n` +
    `总请求数: ${reqs}\n` +
    `P95 延迟: ${(dur['p(95)'] || 0).toFixed(2)} ms\n` +
    `错误率: ${(fail * 100).toFixed(2)} %\n` +
    `新建(201): ${(m.devices_created_201 && m.devices_created_201.values.count) || 0}\n` +
    `复用(200): ${(m.devices_reused_200 && m.devices_reused_200.values.count) || 0}\n`
  );
}
