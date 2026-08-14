# Relay/P2P 跨网络规范

> 版本: 1.0.0
> 最后更新: 2026-07-16
> 状态: 已批准（预留设计）

## 1. 概述

本规范定义 DropVoice 的跨网络传输方案，支持 P2P 直连和 Relay 中继。

**实施时间线:** v2.0.0

## 2. 决策

### 2.1 连接模式

| 模式 | 说明 | 延迟 | 成本 | 成功率 |
|------|------|------|------|--------|
| LAN | 局域网直连 | <1ms | 0 | 99% |
| P2P | NAT 穿透直连 | 10-100ms | 0 | 60-90% |
| Relay | 服务器中继 | 50-200ms | $$$ | 99% |

### 2.2 连接优先级

```mermaid
flowchart LR
    LAN["1. 局域网"] --> P2P["2. P2P"] --> Relay["3. Relay"]
```

### 2.3 P2P 技术选型

| 技术 | 用途 | 说明 |
|------|------|------|
| STUN | 获取公网 IP | 服务器返回客户端公网地址 |
| ICE | NAT 穿透 | 收集候选地址并测试连接 |
| TURN | Relay 中继 | P2P 失败时的兜底方案 |

### 2.4 P2P 成功率

| 场景 | 成功率 | 说明 |
|------|--------|------|
| 家用 ↔ 家用 | 85-95% | 同运营商成功率高 |
| 家用 ↔ 移动 | 60-75% | 移动网络 NAT 严格 |
| 移动 ↔ 移动 | 30-50% | 双方对称 NAT |
| 企业 ↔ 企业 | 10-30% | 企业防火墙限制 |

### 2.5 P2P 失败场景

| 场景 | 原因 | 解决方案 |
|------|------|---------|
| 双方对称 NAT | 端口不可预测 | 走 Relay |
| 防火墙阻止 | 企业/学校网络 | 走 Relay |
| 运营商限制 | 4G/5G 网络 | 走 Relay |

### 2.6 收费模式

| 功能 | Free | Pro ($5/月) | Enterprise |
|------|------|------------|------------|
| 局域网直连 | ✅ 无限制 | ✅ 无限制 | ✅ 无限制 |
| P2P 连接 | ✅ 无限制 | ✅ 无限制 | ✅ 无限制 |
| Relay 流量 | 100 MB/月 | 10 GB/月 | 无限制 |
| Relay 带宽 | 1 MB/s | 10 MB/s | 无限制 |
| 设备数量 | 3 台 | 10 台 | 无限制 |

### 2.7 遥测数据输出

**初期:** 输出到文件（JSON Lines 格式）

**后期:** 接入 OTel Collector

### 2.8 版本规划

| 版本 | 功能 | 时间线 |
|------|------|--------|
| v0.1.0 | 基础功能（局域网直连） | 当前 |
| v0.2.0 | 多设备管理、局域网发现 | 近期 |
| v1.0.0 | 正式发布 | 目标 |
| v1.1.0 | 信令服务器 | 中期 |
| v2.0.0 | P2P/Relay 跨网络 | 远期 |

## 3. 决策依据

### 3.1 选择 P2P + Relay 混合方案

**选择理由:**
- P2P 延迟低、成本低
- Relay 保证 99% 可用性
- 自动选择最佳路径

**否决方案:**
- ❌ 纯 P2P: 成功率不够高
- ❌ 纯 Relay: 成本高、延迟高

### 3.2 P2P 技术选择

**选择理由:**
- STUN/ICE 是业界标准
- TURN 作为 Relay 方案成熟

## 4. 接口规范

### 4.1 连接模式枚举

```rust
// src-tauri/src/network/mode.rs

enum ConnectionMode {
    Lan,
    P2P,
    Relay,
}
```

### 4.2 网络状态

```typescript
// packages/core/src/network/mode.ts

interface NetworkStatus {
  mode: ConnectionMode;
  isOnline: boolean;
  publicIp?: string;
  latency?: number;
}

function useNetworkStatus(): NetworkStatus;
```

### 4.3 用量 API

```typescript
// packages/core/src/billing/api.ts

interface UsageInfo {
  plan: 'free' | 'pro' | 'enterprise';
  relayBytesUsed: number;
  relayBytesLimit: number;
  relayDurationUsed: number;
  relayDurationLimit: number;
}

async function getUsage(): Promise<UsageInfo>;
async function upgradePlan(plan: 'pro' | 'enterprise'): Promise<string>;
```

### 4.4 UsageAlert 组件

```typescript
// apps/mobile/src/components/UsageAlert.tsx

interface UsageAlertProps {
  usage: UsageInfo;
  onUpgrade: () => void;
}
```

### 4.5 STUN 客户端

```rust
// src-tauri/src/network/stun.rs

async fn get_public_address() -> Result<(String, u16)>;
```

### 4.6 P2P 管理器

```rust
// src-tauri/src/network/p2p.rs

struct P2PManager {
    stun_servers: Vec<String>,
    turn_servers: Vec<TurnServer>,
}

impl P2PManager {
    async fn connect_p2p(&self, remote_ip: &str, remote_port: u16) -> Result<TcpStream>;
    async fn connect_via_relay(&self, relay_server: &TurnServer, remote_peer_id: &str) -> Result<TcpStream>;
    async fn connect(&self, remote_ip: &str, remote_port: u16, relay_server: Option<&TurnServer>) -> Result<(TcpStream, ConnectionMode)>;
}
```

## 5. 约束

1. **P2P 优先**: 能 P2P 就不走 Relay
2. **透明切换**: 连接模式切换对用户透明
3. **用量提示**: 接近限额时提醒用户
4. **版本规划**: v2.0.0 实现
5. **遥测输出**: 初期输出到文件，后期接入 OTel
