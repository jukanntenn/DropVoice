// k6 压测脚本：心跳稳态（spec 11 §13.3 场景②）。
// 写入密集场景：PUT /status 触发 batch flush 写磁盘。
//
// 运行：k6 run tests/load/heartbeat.js -e BASE_URL=http://localhost:8080

import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE_URL || 'http://localhost:8080';

export const options = {
  scenarios: {
    heartbeat: {
      executor: 'constant-arrival-rate',
      rate: 1014,
      timeUnit: '1s',
      duration: '1m',
      preAllocatedVUs: 1500,
      maxVUs: 3000,
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<100'],
    http_req_failed: ['rate<0.01'],
  },
};

// setup：注册设备拿 token。
export function setup() {
  const tokens = [];
  const params = {
    headers: { 'Content-Type': 'application/json' },
    insecureSkipTLSVerify: true,
  };
  for (let i = 0; i < 1500; i++) {
    // 生成 UUID v4 格式的 device_id
    const hex = i.toString(16).padStart(12, '0');
    const device_id = `00000000-0000-4000-8000-${hex}`;
    const res = http.post(
      `${BASE}/devices`,
      JSON.stringify({
        device_id,
        platform: 'desktop',
        address: { ip: `10.0.${Math.floor(i / 254)}.${(i % 254) + 1}`, port: 38425 },
      }),
      params
    );
    if (res.status === 201) {
      tokens.push({ device_id, token: res.json('pairing_token') });
    }
  }
  if (tokens.length === 0) {
    throw new Error('setup failed: no devices registered');
  }
  return { tokens };
}

export default function (data) {
  const idx = __VU % data.tokens.length;
  const { device_id, token } = data.tokens[idx];

  const res = http.put(
    `${BASE}/devices/${device_id}/status`,
    JSON.stringify({
      address: { ip: `10.0.${Math.floor(idx / 254)}.${(idx % 254) + 1}`, port: 38425 },
    }),
    {
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      insecureSkipTLSVerify: true,
    }
  );

  check(res, {
    'status is 200': (r) => r.status === 200,
  });
}
