// Events 订阅封装
// 对应 Rust 侧 events.rs 的事件名

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/** 事件名常量（与 Rust events.rs 保持一致） */
export const EVENT = {
  NOTE_CREATED: 'note:created',
  NOTE_UPDATED: 'note:updated',
  NOTE_DELETED: 'note:deleted',
  NOTE_PINNED: 'note:pinned',
  NOTE_UNPINNED: 'note:unpinned',
  NOTE_COMPLETED: 'note:completed',
  NOTE_UNCOMPLETED: 'note:uncompleted',
  CONFIG_UPDATED: 'config:updated',
  TRAY_NEW_NOTE: 'tray:new-note',
  STORAGE_ERROR: 'storage:error',
} as const;

/** 便签事件 payload */
interface NotePayload {
  id: string;
}

/** 订阅 note:updated */
export function onNoteUpdated(cb: (id: string) => void): Promise<UnlistenFn> {
  return listen<NotePayload>(EVENT.NOTE_UPDATED, (e) => cb(e.payload.id));
}

/** 订阅 note:deleted */
export function onNoteDeleted(cb: (id: string) => void): Promise<UnlistenFn> {
  return listen<NotePayload>(EVENT.NOTE_DELETED, (e) => cb(e.payload.id));
}

/** 订阅 note:unpinned */
export function onNoteUnpinned(cb: (id: string) => void): Promise<UnlistenFn> {
  return listen<NotePayload>(EVENT.NOTE_UNPINNED, (e) => cb(e.payload.id));
}

/** 订阅 note:completed */
export function onNoteCompleted(cb: (id: string) => void): Promise<UnlistenFn> {
  return listen<NotePayload>(EVENT.NOTE_COMPLETED, (e) => cb(e.payload.id));
}

/** 订阅 note:uncompleted */
export function onNoteUncompleted(cb: (id: string) => void): Promise<UnlistenFn> {
  return listen<NotePayload>(EVENT.NOTE_UNCOMPLETED, (e) => cb(e.payload.id));
}

/** 订阅 config:updated */
export function onConfigUpdated(cb: () => void): Promise<UnlistenFn> {
  return listen(EVENT.CONFIG_UPDATED, () => cb());
}

/** 订阅 tray:new-note（托盘菜单/全局快捷键触发新建便签） */
export function onTrayNewNote(cb: () => void): Promise<UnlistenFn> {
  return listen(EVENT.TRAY_NEW_NOTE, () => cb());
}

/** 订阅 storage:error（落盘失败，主窗口显示 toast） */
export function onStorageError(cb: (message: string) => void): Promise<UnlistenFn> {
  return listen<string>(EVENT.STORAGE_ERROR, (e) => cb(e.payload));
}

/** 一次性订阅所有便签事件，统一回调
 * 返回 unlisten 函数（在 onDestroy 时调用）
 */
export async function subscribeAllNotes(
  handler: (event: string, id: string) => void,
): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    listen<NotePayload>(EVENT.NOTE_CREATED, (e) => handler(EVENT.NOTE_CREATED, e.payload.id)),
    listen<NotePayload>(EVENT.NOTE_UPDATED, (e) => handler(EVENT.NOTE_UPDATED, e.payload.id)),
    listen<NotePayload>(EVENT.NOTE_DELETED, (e) => handler(EVENT.NOTE_DELETED, e.payload.id)),
    listen<NotePayload>(EVENT.NOTE_PINNED, (e) => handler(EVENT.NOTE_PINNED, e.payload.id)),
    listen<NotePayload>(EVENT.NOTE_UNPINNED, (e) => handler(EVENT.NOTE_UNPINNED, e.payload.id)),
    listen<NotePayload>(EVENT.NOTE_COMPLETED, (e) => handler(EVENT.NOTE_COMPLETED, e.payload.id)),
    listen<NotePayload>(EVENT.NOTE_UNCOMPLETED, (e) => handler(EVENT.NOTE_UNCOMPLETED, e.payload.id)),
  ]);

  // 返回一个聚合的 unlisten
  return () => unlisteners.forEach((u) => u());
}
