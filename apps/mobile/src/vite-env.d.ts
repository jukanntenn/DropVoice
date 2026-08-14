/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** 信令服务器 API 基础 URL（§2.1）。dev 留空走 Vite proxy，prod 填同域域名。 */
  readonly VITE_API_BASE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
