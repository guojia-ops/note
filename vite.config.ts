import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'node:path';

// Tauri 多入口配置：主窗口 + 便签窗口
// 每个 Tauri 窗口对应一个独立 HTML 入口
export default defineConfig({
  plugins: [svelte()],

  // Tauri 期望的构建产物结构
  build: {
    target: 'es2021',
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'src/index.html'),
        note: resolve(__dirname, 'src/note.html'),
      },
    },
  },

  // 开发服务器配置（Tauri dev 模式）
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
  },

  // Tauri 环境变量
  envPrefix: ['VITE_', 'TAURI_'],

  // 路径别名（与 tsconfig.json paths 对齐）
  resolve: {
    alias: {
      $lib: resolve(__dirname, './src/lib'),
      $stores: resolve(__dirname, './src/stores'),
    },
  },
});
