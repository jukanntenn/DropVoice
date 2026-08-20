import path from 'node:path';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// 静态 marketing 站：无 API 代理，纯静态构建产物，部署目标独立决定。
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    host: true,
    port: 5175,
  },
  build: {
    rollupOptions: {
      output: {
        // 营销页首屏优先：框架层单独成块，长缓存；动效库与 UI 包次之。
        manualChunks: {
          react: ['react', 'react-dom'],
          motion: ['framer-motion'],
        },
      },
    },
  },
});
