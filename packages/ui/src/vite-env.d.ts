/**
 * Ambient Vite env types for `import.meta.env`.
 *
 * Declared locally so this package typechecks without requiring `vite/client`
 * to be resolvable from the workspace package. In consuming Vite apps the
 * canonical `vite/client` types augment these via interface merging.
 */
interface ImportMetaEnv {
  readonly DEV?: boolean;
  readonly PROD?: boolean;
  readonly MODE?: string;
  readonly SSR?: boolean;
  readonly BASE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

/**
 * Ambient declarations for side-effect CSS imports (e.g. @fontsource weights),
 * so `import '@fontsource/inter/400.css'` typechecks. Vite handles resolution.
 */
declare module '*.css';

/**
 * §2：品牌 logo 模块 import（`import logoSrc from '../assets/logo.png'`）。
 * 默认导出为 Vite emit 的哈希资产 URL（string）。沿用本文件"不依赖 vite/client"
 * 的约定，显式声明而非 reference vite/client。
 */
declare module '*.png' {
  const src: string;
  export default src;
}
