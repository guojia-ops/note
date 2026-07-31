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

/** 应用主题到 <html> data-theme 属性
 * light / dark 直接生效；system 跟随 prefers-color-scheme
 */
export function applyTheme(theme: Config['theme']): void {
  const html = document.documentElement;
  if (theme === 'system') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    html.setAttribute('data-theme', prefersDark ? 'dark' : 'light');

    // 监听系统主题变化（仅 system 模式）
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
      if (document.documentElement.getAttribute('data-theme') === 'system') return;
      // 当前是 system 模式，跟随系统更新
      html.setAttribute('data-theme', e.matches ? 'dark' : 'light');
    });
  } else {
    html.setAttribute('data-theme', theme);
  }
}
