/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** 桌面 webview 信令服务器 URL（§5.3 CSP / SSE 连接）。dev 默认 localhost:38424。 */
  readonly VITE_SIGNALING_BASE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
