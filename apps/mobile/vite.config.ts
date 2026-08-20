import path from 'node:path';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// 开发 / 本地验收：/api 反代到 pairing-server，PWA 走同源。
// 默认直连裸后端（cargo run，http://localhost:38424）。容器验收
// （pnpm accept:up，HTTP :8080）时覆盖 target —— .vscode/tasks.json 的
// accept:mobile task 已内置：
//   VITE_API_PROXY_TARGET=http://localhost:8080
const apiProxyTarget = process.env.VITE_API_PROXY_TARGET ?? 'http://localhost:38424';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    // 监听所有接口便于局域网内手机访问。
    host: true,
    port: 5174,
    // §2.1 dev: /api 反代到 pairing-server，PWA 走同源。
    proxy: {
      '/api': {
        target: apiProxyTarget,
        changeOrigin: true,
        // 容器验收 target 是自签 HTTPS，需跳过证书校验。
        secure: !apiProxyTarget.startsWith('https'),
      },
    },
  },
});
