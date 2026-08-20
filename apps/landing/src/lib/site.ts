/**
 * 站点级常量——单一来源。
 *
 * 事实行的值必须真实可验证（设计原则 3：诚实文案）。
 * version 与根 package.json 同步（pnpm version:* 后随发版更新）。
 */
export const SITE = {
  repo: 'https://github.com/jukanntenn/DropVoice',
  /** GitHub 最新 release 页（各平台安装包都在这里，永不失效的直链策略）。 */
  releases: 'https://github.com/jukanntenn/DropVoice/releases/latest',
  /** 手机 PWA 入口（生产 pairing-server 同域反代；扫码目标）。 */
  pwaUrl: 'https://api.dropvoice.app',
  version: '0.2.0',
  license: 'MIT',
} as const;

/** 桌面端平台下载入口（终章 CTA 三按钮共用 release 页）。 */
export const PLATFORM_LINKS = [
  { label: 'Windows', href: SITE.releases },
  { label: 'macOS', href: SITE.releases },
  { label: 'Linux', href: SITE.releases },
] as const;
