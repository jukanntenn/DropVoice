# 测试策略规范

> 版本: 1.0.0
> 最后更新: 2026-07-16
> 状态: 已批准

## 1. 概述

本规范定义 DropVoice 的测试分层、Mock 策略和覆盖率目标。

## 2. 决策

### 2.1 测试金字塔

```mermaid
pyramid
    title 测试金字塔
    "E2E 测试" : 10
    "集成测试" : 30
    "单元测试" : 60
```

### 2.2 测试分层

| 层级 | 工具 | Mock 策略 | 覆盖目标 |
|------|------|----------|---------|
| L1 单元测试 | Vitest + cargo test | Mock 边界依赖 | 90%+ |
| L2 组件测试 | @testing-library/react | Mock Tauri API | 70%+ |
| L3 集成测试 | Vitest + MSW | 仅 Mock Tauri IPC | 60%+ |
| L4 E2E 测试 | Playwright | 不 Mock，真实服务 | 关键路径 |

### 2.3 Mock 策略: 最小 Mock

**原则:** Mock 粒度最小，尽可能测试真实行为。

| 测试类型 | Mock 什么 | 不 Mock 什么 |
|---------|----------|-------------|
| 单元测试 | localStorage、WebSocket、Tauri API | 被测函数内部逻辑 |
| 组件测试 | Tauri IPC | 组件交互 |
| 集成测试 | 仅 Tauri IPC | WebSocket、业务逻辑 |
| E2E 测试 | ✅ 不 Mock | 真实应用、真实用户操作 |

### 2.4 前端测试

#### 测试目录结构

```
apps/desktop/src/
├── hooks/
│   └── useConnection.test.ts     # 共置测试
├── components/
│   └── Button.test.tsx           # 共置测试
└── machines/
    └── connection.test.ts        # 状态机测试

apps/mobile/src/
├── hooks/
│   └── useDeviceManager.test.ts
└── components/
    └── DeviceDots.test.tsx

tests/                            # 共享测试配置
├── setup.ts                      # 测试环境初始化
├── mocks/
│   ├── tauri.ts                  # Tauri API mock
│   └── websocket.ts              # WebSocket mock
└── utils/
    └── render.tsx                 # 自定义 render
```

#### Vitest 配置

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    globals: true,
    coverage: {
      reporter: ['text', 'lcov'],
      exclude: ['node_modules/', 'tests/'],
    },
  },
});
```

#### 测试设置文件

```typescript
// tests/setup.ts
import '@testing-library/jest-dom';
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

// 每个测试后清理
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock WebSocket
global.WebSocket = vi.fn().mockImplementation(() => ({
  send: vi.fn(),
  close: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  readyState: WebSocket.OPEN,
}));
```

### 2.5 Rust 测试

#### 测试目录结构

```
apps/desktop/src-tauri/
├── src/
│   ├── error.rs
│   │   └── #[cfg(test)] mod tests   # 共置单元测试
│   └── machines/
│       └── connection.rs
│           └── #[cfg(test)] mod tests
└── tests/                            # 集成测试
    ├── server_tests.rs
    ├── websocket_tests.rs
    └── text_injection_tests.rs
```

### 2.6 E2E 测试

**可选**，用于关键用户流程验证。

```
e2e/
├── playwright.config.ts
├── connection.spec.ts            # 连接流程
└── send-text.spec.ts             # 发送文本流程
```

## 3. 决策依据

### 3.1 测试工具选择

**参考项目:**
- CC-Switch: Vitest + @testing-library/react + MSW
- Yaak: Vitest

**选择理由:**
- Vitest 快速、ESM 原生支持
- @testing-library/react 测试用户行为
- MSW 模拟 API 响应

### 3.2 Mock 策略: 最小 Mock

**选择理由:**
- Mock 粒度最小，尽可能测试真实行为
- 发现集成问题
- 测试更可靠

**否决方案:**
- ❌ 大量 Mock: 测试不可靠，无法发现集成问题

### 3.3 E2E 测试: 真实服务

**选择理由:**
- E2E 测试的核心价值是模拟真实用户场景
- 启动真实 Tauri 应用 + 真实 WebSocket 连接
- 发现集成问题

**否决方案:**
- ❌ Mock E2E: 失去 E2E 测试的意义

## 4. 接口规范

### 4.1 测试脚本

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "test:rust": "cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml",
    "test:all": "pnpm test && pnpm test:rust"
  }
}
```

### 4.2 测试覆盖率目标

| 层级 | 目标 |
|------|------|
| 工具函数 | 90%+ |
| 状态机 | 80%+ |
| React 组件 | 70%+ |
| Rust 核心逻辑 | 80%+ |
| 整体 | 60%+ |

### 4.3 测试文件命名

- 前端: `*.test.ts` / `*.test.tsx`
- Rust 单元测试: `#[cfg(test)] mod tests` 或 `*_tests.rs`
- Rust 集成测试: `tests/*.rs`

### 4.4 测试示例

#### 状态机测试

```typescript
// packages/core/src/machines/connection.test.ts
import { describe, it, expect } from 'vitest';
import { connectionReducer } from './connection';

describe('connectionReducer', () => {
  it('should transition from idle to connecting', () => {
    const state = { status: 'idle' as const };
    const next = connectionReducer(state, {
      type: 'CONNECT',
      url: 'ws://192.168.1.1:38425/ws'
    });

    expect(next).toEqual({
      status: 'connecting',
      url: 'ws://192.168.1.1:38425/ws',
      attempt: 0,
    });
  });

  it('should not allow SEND from idle state', () => {
    const state = { status: 'idle' as const };
    const next = connectionReducer(state, { type: 'SEND', text: 'hello' });

    expect(next.status).toBe('idle');
  });
});
```

#### 组件测试

```typescript
// apps/mobile/src/components/Button.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from './Button';

describe('Button', () => {
  it('renders children', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Click</Button>);

    fireEvent.click(screen.getByText('Click'));
    expect(handleClick).toHaveBeenCalledOnce();
  });
});
```

## 5. 约束

1. **测试独立性**: 每个测试必须独立运行，不依赖其他测试
2. **测试可重复**: 测试结果必须一致，不依赖外部状态
3. **测试快速**: 单元测试必须在几秒内完成
4. **覆盖率门控**: CI 中覆盖率低于阈值则失败
5. **E2E 可选**: E2E 测试用于关键路径，不强制所有场景
