import { defineConfig } from '@playwright/test';

/**
 * Playwright 配置（规范 06 section 2.6）。
 *
 * - 仅使用 chromium（桌面端 Tauri 内嵌 WebView 基于 Chromium）。
 * - webServer 自动启动移动端 Vite 开发服务器（端口 5174）。
 * - 视口设置为移动端尺寸（375×667），模拟真实手机场景。
 */
export default defineConfig({
  testDir: '.',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    baseURL: 'http://localhost:5174',
    trace: 'on-first-retry',
    viewport: { width: 375, height: 667 },
  },
  projects: [
    {
      name: 'chromium',
      use: { browserName: 'chromium' },
    },
  ],
  webServer: {
    command: 'pnpm --filter @dropvoice/mobile dev',
    url: 'http://localhost:5174',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
