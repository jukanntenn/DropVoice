# 版本号与变更日志规范

> 版本: 1.0.0
> 最后更新: 2026-07-16
> 状态: 已批准

## 1. 概述

本规范定义 DropVoice 的版本号规范和变更日志格式。

## 2. 决策

### 2.1 版本号规范

采用 **语义化版本号（Semantic Versioning）**:

```
MAJOR.MINOR.PATCH
  │      │     │
  │      │     └── 修复版本: Bug 修复、小改进
  │      └──────── 次版本: 新功能、向后兼容
  └─────────────── 主版本: 破坏性变更
```

### 2.2 版本号规则

| 变更类型 | 版本变化 | 示例 |
|---------|---------|------|
| Bug 修复 | `0.1.0` → `0.1.1` | 修复连接超时 |
| 新功能 | `0.1.0` → `0.2.0` | 添加多设备支持 |
| 破坏性变更 | `0.9.0` → `1.0.0` | 正式发布 |
| 预发布 | `1.0.0-alpha.1` | 内测版本 |

### 2.3 特殊版本

| 版本 | 含义 | 说明 |
|------|------|------|
| `0.x.x` | 开发阶段 | API 不稳定 |
| `1.0.0` | 正式发布 | API 稳定 |
| `x.y.z-alpha.n` | Alpha | 内部测试 |
| `x.y.z-beta.n` | Beta | 公开测试 |
| `x.y.z-rc.n` | Release Candidate | 候选发布 |

### 2.4 版本规划

| 版本 | 功能 | 时间线 |
|------|------|--------|
| v0.1.0 | 基础功能（局域网直连、QR码、文本注入） | 当前 |
| v0.2.0 | 多设备管理、局域网发现、DESIGN.md | 近期 |
| v1.0.0 | 正式发布（完整测试、代码签名、自动更新） | 目标 |
| v1.1.0 | 信令服务器（HTTPS 扫码） | 中期 |
| v2.0.0 | P2P/Relay 跨网络、计费系统 | 远期 |

### 2.5 版本同步

```javascript
// scripts/sync-version.js
// 同步 package.json 版本到 Cargo.toml
const fs = require('fs');
const path = require('path');

const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
const version = packageJson.version;

// 同步到 Cargo.toml
const cargoPath = path.join('apps/desktop/src-tauri', 'Cargo.toml');
let cargoContent = fs.readFileSync(cargoPath, 'utf8');
cargoContent = cargoContent.replace(/^version = ".*"$/m, `version = "${version}"`);
fs.writeFileSync(cargoPath, cargoContent);
```

### 2.6 版本脚本

```json
{
  "scripts": {
    "version:patch": "pnpm version patch && pnpm version:sync",
    "version:minor": "pnpm version minor && pnpm version:sync",
    "version:major": "pnpm version major && pnpm version:sync",
    "version:sync": "node scripts/sync-version.js",
    "version:pre:alpha": "pnpm version prerelease --preid=alpha",
    "version:pre:beta": "pnpm version prerelease --preid=beta"
  }
}
```

## 3. CHANGELOG 规范

### 3.1 格式

采用 **Keep a Changelog** 格式:

```markdown
# Changelog

本文件记录 DropVoice 的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增
- P2P 跨网络连接支持

### 变更
- 升级到 React 19

### 修复
- (无)

## [0.2.0] - 2026-07-15

### 新增
- 多设备管理功能
- 局域网自动发现
- DESIGN.md 设计规范

### 变更
- 迁移到 axum HTTP 框架
- 统一错误处理规范

### 修复
- 修复 Windows 文本注入丢失字符问题
- 修复 macOS 无障碍权限提示

### 移除
- 移除旧版 WebSocket 实现

## [0.1.0] - 2026-06-01

### 新增
- 初始版本发布
- QR 码扫描连接
- WebSocket 文本传输
- 多语言支持（中/英/日）
- 深色模式支持
```

### 3.2 变更类型

| 类型 | 关键词 | 说明 |
|------|--------|------|
| 新增 | `### 新增` | 新功能 |
| 变更 | `### 变更` | 现有功能变更 |
| 弃用 | `### 弃用` | 即将移除的功能 |
| 移除 | `### 移除` | 已移除的功能 |
| 修复 | `### 修复` | Bug 修复 |
| 安全 | `### 安全` | 安全相关变更 |

### 3.3 提交规范

采用 **Conventional Commits**:

| 提交类型 | 说明 | CHANGELOG 分类 |
|---------|------|---------------|
| `feat:` | 新功能 | 新增 |
| `fix:` | Bug 修复 | 修复 |
| `docs:` | 文档变更 | - |
| `style:` | 代码格式 | - |
| `refactor:` | 重构 | 变更 |
| `perf:` | 性能优化 | 变更 |
| `test:` | 测试 | - |
| `chore:` | 构建/工具 | - |
| `revert:` | 回滚 | 移除 |

### 3.4 自动化

```json
{
  "scripts": {
    "changelog": "conventional-changelog -p angular -i CHANGELOG.md -s",
    "changelog:first": "conventional-changelog -p angular -i CHANGELOG.md -s -r 0"
  },
  "devDependencies": {
    "conventional-changelog-cli": "^5.0.0"
  }
}
```

### 3.5 提交规范配置

```json
// .commitlintrc.json
{
  "extends": ["@commitlint/config-conventional"],
  "rules": {
    "type-enum": [
      2,
      "always",
      ["feat", "fix", "docs", "style", "refactor", "perf", "test", "chore", "revert"]
    ]
  }
}
```

### 3.6 示例提交

```bash
git commit -m "feat: 添加多设备管理功能"
git commit -m "fix: 修复 Windows 文本注入丢失字符问题"
git commit -m "refactor: 迁移到 axum HTTP 框架"
```

## 4. 决策依据

### 4.1 选择语义化版本号

**选择理由:**
- 业界标准
- 清晰表达变更性质
- 自动化工具支持

### 4.2 选择 Keep a Changelog

**选择理由:**
- 格式清晰
- 社区广泛采用
- 机器可读

### 4.3 选择 Conventional Commits

**选择理由:**
- 自动生成 CHANGELOG
- 自动确定版本号
- 清晰的提交历史

## 5. 接口规范

### 5.1 版本脚本

```json
{
  "scripts": {
    "version:patch": "pnpm version patch && pnpm version:sync",
    "version:minor": "pnpm version minor && pnpm version:sync",
    "version:major": "pnpm version major && pnpm version:sync",
    "version:sync": "node scripts/sync-version.js"
  }
}
```

### 5.2 发布流程

```bash
# 1. 更新版本号
pnpm version:patch

# 2. 提交版本变更
git add .
git commit -m "release: v0.1.1"

# 3. 创建标签
git tag v0.1.1

# 4. 推送触发 CI
git push --follow-tags
```

## 6. 约束

1. **版本号必须**: 每次发布必须更新版本号
2. **CHANGELOG 必须**: 每次发布必须更新 CHANGELOG
3. **提交规范**: 提交信息必须遵循 Conventional Commits
4. **版本同步**: package.json 和 Cargo.toml 版本必须同步
5. **标签必须**: 每次发布必须创建 git 标签
