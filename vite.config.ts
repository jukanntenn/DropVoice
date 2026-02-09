import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "path";

export default defineConfig(async () => ({
  plugins: [tailwindcss(), react()],
  clearScreen: false,
  server: {
    host: true,
    port: 5173,
    strictPort: true,
    cors: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        mobile: resolve(__dirname, "src/mobile/index.html"),
      },
    },
  },
}));
