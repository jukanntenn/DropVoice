# 国际化规范

> 版本: 1.0.0
> 最后更新: 2026-07-16
> 状态: 已批准

## 1. 概述

本规范定义 DropVoice 的国际化（i18n）架构、支持语言和翻译工作流。

## 2. 决策

### 2.1 支持语言

| 语言代码 | 语言 | 优先级 | 说明 |
|---------|------|--------|------|
| `en` | English | ⭐⭐⭐⭐⭐ | 默认语言 |
| `zh` | 简体中文 | ⭐⭐⭐⭐⭐ | 主要语言 |
| `zh-TW` | 繁體中文 | ⭐⭐⭐ | 次要语言 |
| `ja` | 日本語 | ⭐⭐⭐ | 次要语言 |

### 2.2 目录结构

```
packages/i18n/
├── src/
│   ├── index.ts                   # 入口
│   ├── config.ts                  # i18n 配置
│   ├── detector.ts                # 语言检测
│   └── types.ts                   # 类型安全的翻译 key
├── locales/
│   ├── en/
│   │   ├── common.json            # 通用翻译
│   │   ├── errors.json            # 错误消息
│   │   ├── devices.json           # 设备相关
│   │   └── settings.json          # 设置相关
│   ├── zh/
│   │   └── ...（同 en 结构）
│   ├── zh-TW/
│   │   └── ...
│   └── ja/
│       └── ...
└── package.json
```

### 2.3 翻译文件结构

```json
// common.json
{
  "app": {
    "name": "DropVoice",
    "description": "Send text from mobile to desktop",
    "loading": "Loading...",
    "error": "Something went wrong"
  },
  "actions": {
    "send": "Send",
    "cancel": "Cancel",
    "confirm": "Confirm",
    "copy": "Copy",
    "paste": "Paste",
    "delete": "Delete",
    "retry": "Retry",
    "close": "Close"
  },
  "status": {
    "connected": "Connected",
    "disconnected": "Disconnected",
    "connecting": "Connecting...",
    "sending": "Sending..."
  }
}
```

```json
// errors.json
{
  "serverAlreadyRunning": "Server is already running",
  "portInUse": "Port {{port}} is already in use",
  "networkUnreachable": "Network is unreachable: {{reason}}",
  "connectionRefused": "Connection refused",
  "connectionTimeout": "Connection timed out",
  "textTooLong": "Text is too long ({{length}}/{{max}} characters)",
  "textInjectionFailed": "Failed to inject text: {{reason}}",
  "suggestion": {
    "checkNetwork": "Please check your network connection",
    "checkServerRunning": "Please ensure the server is running",
    "retryLater": "Please try again later",
    "changePort": "Try using a different port"
  }
}
```

```json
// devices.json
{
  "title": "Devices",
  "addDevice": "Add Device",
  "removeDevice": "Remove Device",
  "selectTarget": "Select target device",
  "emptyState": "No devices connected",
  "maxLimit": "Maximum devices reached",
  "invalidUrl": "Invalid connection URL",
  "cameraRequiresHTTPS": "Camera requires HTTPS connection",
  "cameraNotAvailable": "Camera not available",
  "manualInput": "Enter URL manually",
  "connectionCode": "Connection code"
}
```

```json
// settings.json
{
  "title": "Settings",
  "language": "Language",
  "theme": "Theme",
  "themeLight": "Light",
  "themeDark": "Dark",
  "themeSystem": "System",
  "inputDelay": "Input delay (ms)",
  "minimizeToTray": "Minimize to tray"
}
```

### 2.4 类型安全

```typescript
// packages/i18n/src/types.ts

import type en from '../locales/en';

type FlattenKeys<T, Prefix extends string = ''> = T extends object
  ? {
      [K in keyof T]: K extends string
        ? T[K] extends object
          ? FlattenKeys<T[K], `${Prefix}${K}.`>
          : `${Prefix}${K}`
        : never;
    }[keyof T]
  : never;

export type TranslationKey = FlattenKeys<typeof en>;
```

### 2.5 i18n 配置

> **实现说明 (2026-07-21):** 实际采用 **4 命名空间**（`common` / `devices`
> / `errors` / `settings`）而非单 `translation` 命名空间。这样每个 JSON
> 文件按功能模块拆分，便于维护（§2.2 目录结构）。调用方使用
> `t('common:app.name')` / `t('errors:portInUse', { port })` 等带前缀的 key。
> 必须在 `init()` 中显式设置 `ns` 和 `defaultNS: 'common'`，否则 i18next
> 无法解析带命名空间前缀的 key。下方代码已反映实际配置。

```typescript
// packages/i18n/src/config.ts

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import enCommon from '../locales/en/common.json';
import enDevices from '../locales/en/devices.json';
import enErrors from '../locales/en/errors.json';
import enSettings from '../locales/en/settings.json';
// ...其余语言同构导入

export const resources = {
  en: { common: enCommon, devices: enDevices, errors: enErrors, settings: enSettings },
  // zh / zh-TW / ja 同结构
} as const;

export const NAMESPACES = ['common', 'devices', 'errors', 'settings'] as const;
export const DEFAULT_LANGUAGE = 'en';

export function initI18n() {
  return i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
      resources,
      fallbackLng: DEFAULT_LANGUAGE,
      supportedLngs: SUPPORTED_LANGUAGES,
      ns: NAMESPACES,                 // 显式注册全部命名空间
      defaultNS: 'common',            // 无前缀的 key 从 common 解析
      interpolation: {
        escapeValue: false,
      },
      detection: {
        order: ['querystring', 'localStorage', 'navigator'],
        lookupQuerystring: 'lang',
        lookupLocalStorage: 'dropvoice-lang',
      },
    });
}
```

### 2.6 Tauri 后端 i18n

```rust
// src-tauri/src/i18n.rs

#[derive(Debug, Deserialize)]
struct Translations {
    errors: HashMap<String, String>,
    tray: TrayTranslations,
}

#[derive(Debug, Deserialize)]
struct TrayTranslations {
    show: String,
    hide: String,
    quit: String,
}

fn load_translations(lang: &str) -> Translations;
fn validate_language(lang: &str) -> Result<&str, String>;
fn get_system_language() -> String;
```

### 2.7 翻译工作流

```json
{
  "scripts": {
    "i18n:extract": "i18next-parser 'src/**/*.{ts,tsx}' -o packages/i18n/locales/en",
    "i18n:validate": "node scripts/validate-translations.js"
  }
}
```

## 3. 决策依据

### 3.1 选择 i18next

**选择理由:**
- 业界标准
- React 集成良好（react-i18next）
- 支持复数、上下文等高级功能

### 3.2 按功能模块拆分翻译文件

**选择理由:**
- 便于维护
- 避免单文件过大
- 按需加载

### 3.3 类型安全的翻译 key

**选择理由:**
- 编译时检查翻译 key 是否存在
- IDE 自动补全
- 减少运行时错误

## 4. 接口规范

### 4.1 useTranslation Hook

```typescript
import { useTranslation } from 'react-i18next';

const { t, i18n } = useTranslation();

// 使用
t('app.name');
t('errors.portInUse', { port: 38425 });
```

### 4.2 语言检测

```typescript
// 检测顺序: querystring > localStorage > navigator
{
  detection: {
    order: ['querystring', 'localStorage', 'navigator'],
    lookupQuerystring: 'lang',
    lookupLocalStorage: 'dropvoice-lang',
  }
}
```

### 4.3 语言切换

```typescript
// 前端
i18n.changeLanguage('zh');

// Tauri command
invoke('set_language', { lang: 'zh' });
```

## 5. 约束

1. **翻译完整性**: 所有 key 必须在所有语言中存在
2. **参数化**: 动态内容使用 `{{param}}` 占位符
3. **类型安全**: 翻译 key 必须有 TypeScript 类型
4. **回退语言**: 英语作为回退语言
5. **Tauri 后端**: 系统托盘菜单等必须支持 i18n
