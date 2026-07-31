// Notes store
// PRD 12.6: 前端订阅事件后刷新本地状态

import { writable } from 'svelte/store';
import type { Note } from '../types/note';
import { getNotes } from '../lib/commands';
import { subscribeAllNotes, EVENT } from '../lib/events';

/** Notes store 状态 */
export const notes = writable<Note[]>([]);

/** 加载标志 */
export const notesLoading = writable<boolean>(true);

/** 从后端刷新所有便签 */
export async function refreshNotes(): Promise<void> {
  try {
    const list = await getNotes();
    // 按更新时间倒序（PRD 7 默认排序）
    list.sort((a, b) => b.updated_at - a.updated_at);
    notes.set(list);
  } finally {
    notesLoading.set(false);
  }
}

let unsub: (() => void) | null = null;

/** 启动事件订阅（在主窗口 onMount 调用）
 * 收到任何便签事件后，统一 refresh 全量数据
 * 全量 refresh 简单可靠，数据量小（PRD 性能指标 ≤200ms for 1000 条）
 */
export async function startNotesSubscription(): Promise<void> {
  if (unsub) return; // 防止重复订阅

  unsub = await subscribeAllNotes(async (_event, _id) => {
    await refreshNotes();
  });
}

/** 停止事件订阅（在 onDestroy 调用） */
export function stopNotesSubscription(): void {
  if (unsub) {
    unsub();
    unsub = null;
  }
}

export { EVENT };
