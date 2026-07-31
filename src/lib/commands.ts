// Commands 封装：类型安全的 invoke 调用
// 对应 Rust 侧 commands.rs 的 11 个命令

import { invoke } from '@tauri-apps/api/core';
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

/** 贴出便签（创建便签窗口） */
export function pinNote(id: string): Promise<void> {
  return invoke<void>('pin_note', { id });
}

/** 收回便签（关闭便签窗口） */
export function unpinNote(id: string): Promise<void> {
  return invoke<void>('unpin_note', { id });
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
