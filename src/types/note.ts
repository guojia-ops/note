// 前端类型定义
// 与 Rust 侧 types.rs 对齐（serde 序列化后的 JSON 形态）

/** 便签颜色（6 色板，PRD 9.3） */
export type NoteColor = 'yellow' | 'pink' | 'green' | 'blue' | 'purple' | 'cream';

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
  /** 桌面贴出时的随机微旋转角度（-2.0 ~ 2.0，默认 0.0）
   *  仅作用于桌面便签窗口，主窗口卡片墙不旋转
   *  旧 data.json 缺失该字段时由 Rust serde default 补 0 */
  rotation?: number;
  /** 所属显示器标识（多屏记忆），空字符串回退主屏
   *  旧 data.json 缺失该字段时由 Rust serde default 补 "" */
  monitor?: string;
  /** 完成时间戳（Unix ms），undefined = 未完成，number = 已完成
   *  旧 data.json 缺失该字段时由 Rust serde default 补 undefined */
  completed_at?: number;
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
  /** 便签始终置顶开关
   *  旧 config.json 缺失时由 Rust serde default 补 true */
  always_on_top?: boolean;
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
  /** 所属显示器标识（多屏记忆） */
  monitor?: string;
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
  'cream',
];
