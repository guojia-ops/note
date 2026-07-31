// 前端类型定义
// 与 Rust 侧 types.rs 对齐（serde 序列化后的 JSON 形态）

/** 便签颜色（6 色板，PRD 9.3） */
export type NoteColor = 'yellow' | 'pink' | 'green' | 'blue' | 'purple' | 'orange';

/** 主题（PRD 设置项 7） */
export type Theme = 'light' | 'dark' | 'system';

/** 关闭主窗口行为（PRD 设置项 12） */
export type CloseAction = 'tray' | 'quit';

/** 便签实体（对应 Rust Note） */
export interface Note {
  id: string;
  title: string;
  content: string;
  color: NoteColor;
  width: number;
  height: number;
  x: number;
  y: number;
  created_at: number;
  updated_at: number;
  pinned: boolean;
}

/** 应用配置（对应 Rust Config） */
export interface Config {
  auto_start: boolean;
  hotkey: string;
  default_color: NoteColor;
  auto_backup: boolean;
  backup_keep: number;
  theme: Theme;
  close_action: CloseAction;
}

/** 更新便签的部分字段（对应 Rust UpdateNoteFields） */
export interface UpdateNoteFields {
  title?: string;
  content?: string;
  color?: NoteColor;
  width?: number;
  height?: number;
  x?: number;
  y?: number;
}

/** 导入结果（对应 Rust ImportResult） */
export interface ImportResult {
  imported: number;
  skipped: number;
}

/** 6 色板枚举值（用于遍历） */
export const NOTE_COLORS: NoteColor[] = [
  'yellow',
  'pink',
  'green',
  'blue',
  'purple',
  'orange',
];

/** 颜色 → CSS 变量名映射 */
export const COLOR_TO_CSS_VAR: Record<NoteColor, string> = {
  yellow: '--color-yellow',
  pink: '--color-pink',
  green: '--color-green',
  blue: '--color-blue',
  purple: '--color-purple',
  orange: '--color-orange',
};
