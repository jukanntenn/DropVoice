import path from 'node:path';

import { defineConfig } from 'vitest/config';

export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    // packages/core 测试是纯 TS 逻辑测试，不依赖 DOM；用 node 环境避免引入 jsdom 依赖。
    environment: 'node',
    globals: true,
    include: ['src/**/*.{test,spec}.{ts,tsx}', 'tests/**/*.{test,spec}.{ts,tsx}'],
    coverage: {
      reporter: ['text', 'lcov'],
      exclude: ['node_modules/', 'tests/', '**/*.config.*', 'src/**/index.ts', 'src/**/*.test.ts'],
      // Overall floor per spec 06 §4.2 (overall 60%). Per-category targets
      // (utils 90 / machines 80 / components 70) are aspirational and not
      // enforced as a hard gate yet — coverage is reported so drift is visible.
      thresholds: {
        statements: 60,
        branches: 50,
        functions: 60,
        lines: 60,
      },
    },
  },
});
