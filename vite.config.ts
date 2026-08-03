import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'node:path';

// Tauri 多入口配置：主窗口 + 便签窗口
// 每个 Tauri 窗口对应一个独立 HTML 入口
// root 设为 src/，使 dev server 访问 / 即返回 src/index.html
export default defineConfig({
  plugins: [svelte()],

  // 多入口应用，关闭 SPA history fallback，避免 /note.html 被重写到 /index.html
  appType: 'mpa',

  // dev server 与 build 的根目录：src/
  // 这样 dev 访问 / → src/index.html，访问 /note.html → src/note.html
  root: resolve(__dirname, 'src'),

  // Tauri 期望的构建产物结构
  build: {
    target: 'es2021',
    // root 改为 src 后，outDir 必须用绝对路径，否则会输出到 src/dist
    outDir: resolve(__dirname, 'dist'),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'src/index.html'),
        note: resolve(__dirname, 'src/note.html'),
        settings: resolve(__dirname, 'src/settings.html'),
      },
    },
  },

  // 开发服务器配置（Tauri dev 模式）
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
    // 排除 Rust 编译产物，避免 EBUSY（exe 被 cargo run 锁定无法 watch）
    watch: {
      ignored: ['**/src-tauri/target/**', '**/target/**'],
    },
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
