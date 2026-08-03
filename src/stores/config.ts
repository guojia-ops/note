// Config store
// PRD 12.6: 订阅 config:updated 事件后刷新本地状态

import { writable } from 'svelte/store';
import type { Config } from '../types/note';
import { getConfig } from '../lib/commands';
import { onConfigUpdated } from '../lib/events';

/** 默认配置（与 Rust Config::default 对齐，避免首屏空白） */
const DEFAULT_CONFIG: Config = {
  auto_start: false,
  hotkey: 'Ctrl+Alt+N',
  default_color: 'yellow',
  auto_backup: true,
  backup_keep: 5,
  theme: 'system',
  close_action: 'tray',
};

export const config = writable<Config>(DEFAULT_CONFIG);

/** 从后端拉取最新配置 */
export async function refreshConfig(): Promise<void> {
  const c = await getConfig();
  config.set(c);
  applyTheme(c.theme);
}

let unsub: (() => void) | null = null;

/** 启动 config:updated 订阅 */
export async function startConfigSubscription(): Promise<void> {
  if (unsub) return;
  unsub = await onConfigUpdated(async () => {
    await refreshConfig();
  });
}

export function stopConfigSubscription(): void {
  if (unsub) {
    unsub();
    unsub = null;
  }
}

/** 应用主题到 <html>：
 * - data-theme-mode 存储用户选择（light/dark/system），用于判断是否跟随系统
 * - data-theme 存实际生效主题（light/dark），供 CSS 选择器使用
 * system 模式监听 prefers-color-scheme 变化自动切换
 */
let systemMql: MediaQueryList | null = null;
let systemListener: ((e: MediaQueryListEvent) => void) | null = null;

export function applyTheme(theme: Config['theme']): void {
  const html = document.documentElement;
  html.setAttribute('data-theme-mode', theme);

  if (theme === 'system') {
    const mql = window.matchMedia('(prefers-color-scheme: dark)');
    html.setAttribute('data-theme', mql.matches ? 'dark' : 'light');

    // 先移除旧监听器，避免重复注册（主题切换时会重新调用 applyTheme）
    if (systemMql && systemListener) {
      systemMql.removeEventListener('change', systemListener);
    }
    systemListener = (e: MediaQueryListEvent) => {
      // 仅在 system 模式下跟随系统
      if (html.getAttribute('data-theme-mode') === 'system') {
        html.setAttribute('data-theme', e.matches ? 'dark' : 'light');
      }
    };
    mql.addEventListener('change', systemListener);
    systemMql = mql;
  } else {
    // 切换到固定主题时移除系统监听
    if (systemMql && systemListener) {
      systemMql.removeEventListener('change', systemListener);
      systemMql = null;
      systemListener = null;
    }
    html.setAttribute('data-theme', theme);
  }
}
