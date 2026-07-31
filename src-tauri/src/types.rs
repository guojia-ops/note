// DeskNote 数据模型
// 对应 PRD 第 3 章：数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 便签颜色枚举（6 色板，PRD 9.3）
/// 持久化为字符串，前端用 CSS 变量映射
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NoteColor {
    #[default]
    Yellow,
    Pink,
    Green,
    Blue,
    Purple,
    Orange,
}

impl NoteColor {
    /// 前端 CSS 变量值（与 app.css 中 --color-xxx 对应）
    pub fn as_css_var(&self) -> &'static str {
        match self {
            Self::Yellow => "--color-yellow",
            Self::Pink => "--color-pink",
            Self::Green => "--color-green",
            Self::Blue => "--color-blue",
            Self::Purple => "--color-purple",
            Self::Orange => "--color-orange",
        }
    }
}

/// 便签实体（PRD 9）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// 唯一标识（UUID v4）
    pub id: String,
    /// 标题（可空，空时列表用正文前 30 字预览）
    pub title: String,
    /// 正文（纯文本，支持 URL 自动识别）
    pub content: String,
    /// 颜色
    pub color: NoteColor,
    /// 窗口宽度 px
    pub width: u32,
    /// 窗口高度 px
    pub height: u32,
    /// 窗口屏幕坐标 X（位置记忆，PRD 9.1）
    pub x: i32,
    /// 窗口屏幕坐标 Y
    pub y: i32,
    /// 创建时间戳（Unix ms）
    pub created_at: i64,
    /// 最后修改时间戳（Unix ms）
    pub updated_at: i64,
    /// 是否贴出在桌面（true=窗口存在，false=已收回，PRD 9）
    pub pinned: bool,
}

impl Note {
    /// 创建新便签的工厂方法
    /// 位置默认 (100, 100)，尺寸 240x240，颜色默认黄
    pub fn new() -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: Uuid::new_v4().to_string(),
            title: String::new(),
            content: String::new(),
            color: NoteColor::default(),
            width: 240,
            height: 240,
            x: 100,
            y: 100,
            created_at: now,
            updated_at: now,
            pinned: false,
        }
    }

    /// 更新时间戳（每次修改时调用）
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }
}

impl Default for Note {
    fn default() -> Self {
        Self::new()
    }
}

/// 主题（PRD 设置项 7）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

/// 关闭主窗口行为（PRD 设置项 12）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CloseAction {
    /// 最小化到托盘（默认，PRD 8a）
    #[default]
    Tray,
    /// 退出应用
    Quit,
}

/// 应用配置（持久化到 config.json）
/// 对应 PRD 设置项：1/2/3/6/7/12
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 开机自启（PRD 1）
    pub auto_start: bool,
    /// 全局快捷键（PRD 2，默认 Ctrl+Alt+N）
    pub hotkey: String,
    /// 新建便签默认颜色（PRD 3）
    pub default_color: NoteColor,
    /// 自动备份开关（PRD 6）
    pub auto_backup: bool,
    /// 自动备份保留份数
    pub backup_keep: u32,
    /// 主题（PRD 7）
    pub theme: Theme,
    /// 关闭主窗口行为（PRD 12）
    pub close_action: CloseAction,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_start: false,
            hotkey: "Ctrl+Alt+N".to_string(),
            default_color: NoteColor::default(),
            auto_backup: true,
            backup_keep: 5,
            theme: Theme::default(),
            close_action: CloseAction::default(),
        }
    }
}

/// 持久化数据根结构（data.json）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataFile {
    /// 所有便签
    pub notes: Vec<Note>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_new_has_valid_uuid() {
        let n = Note::new();
        assert!(Uuid::parse_str(&n.id).is_ok());
    }

    #[test]
    fn note_new_defaults() {
        let n = Note::new();
        assert_eq!(n.title, "");
        assert_eq!(n.content, "");
        assert_eq!(n.color, NoteColor::Yellow);
        assert_eq!(n.width, 240);
        assert_eq!(n.height, 240);
        assert_eq!(n.pinned, false);
        assert_eq!(n.created_at, n.updated_at);
    }

    #[test]
    fn note_touch_updates_timestamp() {
        let mut n = Note::new();
        let original = n.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(5));
        n.touch();
        assert!(n.updated_at > original);
    }

    #[test]
    fn color_serialization() {
        assert_eq!(
            serde_json::to_string(&NoteColor::Yellow).unwrap(),
            "\"yellow\""
        );
        let c: NoteColor = serde_json::from_str("\"pink\"").unwrap();
        assert_eq!(c, NoteColor::Pink);
    }

    #[test]
    fn config_default_hotkey() {
        let c = Config::default();
        assert_eq!(c.hotkey, "Ctrl+Alt+N");
        assert_eq!(c.close_action, CloseAction::Tray);
        assert!(c.auto_backup);
        assert_eq!(c.backup_keep, 5);
    }
}
