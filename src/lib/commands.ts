// Commands 封装：类型安全的 invoke 调用
// 对应 Rust 侧 commands.rs 的 11 个命令

import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { Config, ImportResult, Note, NoteColor, UpdateNoteFields } from '../types/note';

/** 创建便签 */
export function createNote(params: {
  title?: string;
  content?: string;
  color?: NoteColor;
}): Promise<Note> {
  return invoke<Note>('create_note', {
    title: params.title ?? null,
    content: params.content ?? null,
    color: params.color ?? null,
  });
}

/** 更新便签（部分字段） */
export function updateNote(id: string, fields: UpdateNoteFields): Promise<Note> {
  return invoke<Note>('update_note', { id, fields });
}

/** 删除便签 */
export function deleteNote(id: string): Promise<void> {
  return invoke<void>('delete_note', { id });
}

/** 获取所有便签 */
export function getNotes(): Promise<Note[]> {
  return invoke<Note[]>('get_notes');
}

/** 贴出便签（更新 pinned 状态 + 前端创建便签窗口）
 * 窗口创建由前端 WebviewWindow API 完成，因 Rust 侧 WebviewUrl::App
 * 对含 query string 的路径在 Windows 上解析失败（PathBuf 不支持 ?）
 */
export async function pinNote(id: string): Promise<void> {
  // 先调 Rust 更新 pinned 状态，返回 note 数据用于设置窗口初始位置尺寸
  const note = await invoke<Note>('pin_note', { id });

  // 前端创建便签窗口，用 note 的位置和尺寸恢复
  const label = `note-${id}`;
  const url = `note.html?id=${id}`;

  const win = new WebviewWindow(label, {
    url,
    title: 'DeskNote',
    decorations: false,
    transparent: true,
    alwaysOnTop: true,
    skipTaskbar: true,
    resizable: true,
    visible: false, // 延迟显示，等 NoteApp 加载完数据后 show，避免空白闪烁
    width: note.width,
    height: note.height,
    x: note.x,
    y: note.y,
  });

  return new Promise((resolve, reject) => {
    win.once('tauri://created', () => resolve());
    win.once('tauri://error', (e) => reject(e));
  });
}

/** 收回便签（关闭便签窗口） */
export async function unpinNote(id: string): Promise<void> {
  await invoke<void>('unpin_note', { id });
  // 前端关闭窗口（Rust 侧也会尝试关闭，双保险）
  const label = `note-${id}`;
  try {
    const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
    const existing = await WebviewWindow.getByLabel(label);
    if (existing) await existing.close();
  } catch {
    // 窗口可能已被 Rust 侧关闭，忽略
  }
}

/** 获取配置 */
export function getConfig(): Promise<Config> {
  return invoke<Config>('get_config');
}

/** 更新配置（部分字段，传 JSON 对象） */
export function updateConfig(partial: Partial<Config>): Promise<Config> {
  return invoke<Config>('update_config', { partial });
}

/** 导出便签到指定文件路径 */
export function exportNotes(path: string): Promise<void> {
  return invoke<void>('export_notes', { path });
}

/** 从指定文件路径导入便签 */
export function importNotes(path: string): Promise<ImportResult> {
  return invoke<ImportResult>('import_notes', { path });
}

/** 获取数据目录路径（只读，用于设置页显示） */
export function getDataDir(): Promise<string> {
  return invoke<string>('get_data_dir');
}

/** 全部收回（SOP 8.11，托盘菜单调用） */
export function retractAllNotes(): Promise<void> {
  return invoke<void>('retract_all_notes');
}

/** 退出应用（SOP 8.12） */
export function quitApp(): Promise<void> {
  return invoke<void>('quit_app');
}

/** 重新注册全局快捷键（SOP 8.9，设置页改快捷键后调用） */
export function registerHotkey(): Promise<void> {
  return invoke<void>('register_hotkey');
}
