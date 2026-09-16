// DeskNote 数据模型
// 对应 PRD 第 3 章：数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 便签颜色枚举（6 色板，PRD 9.3）
/// 持久化为字符串，前端用 CSS 变量映射
/// v2.0：Orange → Cream（奶白便签），serde 兼容旧 "orange" 数据
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoteColor {
    #[default]
    Yellow,
    Pink,
    Green,
    Blue,
    Purple,
    Cream,
}

impl<'de> Deserialize<'de> for NoteColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "yellow" => Ok(NoteColor::Yellow),
            "pink" => Ok(NoteColor::Pink),
            "green" => Ok(NoteColor::Green),
            "blue" => Ok(NoteColor::Blue),
            "purple" => Ok(NoteColor::Purple),
            "cream" | "orange" => Ok(NoteColor::Cream),
            _ => Ok(NoteColor::default()),
        }
    }
}

impl Serialize for NoteColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            NoteColor::Yellow => "yellow",
            NoteColor::Pink => "pink",
            NoteColor::Green => "green",
            NoteColor::Blue => "blue",
            NoteColor::Purple => "purple",
            NoteColor::Cream => "cream",
        })
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
    /// 桌面贴出时的随机微旋转角度（-2.0 ~ 2.0，默认 0.0）
    /// 仅作用于桌面便签窗口，主窗口卡片墙不旋转
    /// serde default：旧 data.json 读取时缺失字段默认 0.0
    #[serde(default)]
    pub rotation: f32,
    /// 所属显示器标识（多屏记忆，v1.1 优化阶段 2.3）
    /// Tauri Monitor.name() 可空，用 String 存；空字符串回退主屏
    /// serde default：旧 data.json 读取时缺失字段默认 ""
    #[serde(default)]
    pub monitor: String,
    /// 完成时间戳（Unix ms），None = 未完成，Some(ts) = 已完成
    /// serde default：旧 data.json 缺失字段默认为 None（未完成）
    #[serde(default)]
    pub completed_at: Option<i64>,
}

impl Note {
    /// 创建新便签的工厂方法
    /// 位置默认 (100, 100)，尺寸 240x240，颜色默认黄
    /// rotation 基于 UUID 字节生成 -0.8..=0.8 随机角度（不引入 rand 依赖）
    pub fn new() -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        let uuid = Uuid::new_v4();
        let rotation = {
            let b = uuid.as_bytes();
            let n = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
            ((n % 1601) as f32) / 1000.0 - 0.8
        };
        Self {
            id: uuid.to_string(),
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
            rotation,
            monitor: String::new(),
            completed_at: None,
        }
    }

    /// 更新时间戳（每次修改时调用）
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp_millis();
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
    /// 便签始终置顶（v1.1 优化阶段 2.1，默认 true）
    /// serde default = true：旧 config.json 缺省保持原行为（始终置顶）
    #[serde(default = "default_true")]
    pub always_on_top: bool,
}

/// serde default helper：布尔字段缺省为 true
fn default_true() -> bool {
    true
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
            always_on_top: true,
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
    fn color_orange_backcompat() {
        // 旧 data.json 中 "orange" 反序列化为 Cream
        let c: NoteColor = serde_json::from_str("\"orange\"").unwrap();
        assert_eq!(c, NoteColor::Cream);
        // 序列化后输出 "cream"
        assert_eq!(
            serde_json::to_string(&NoteColor::Cream).unwrap(),
            "\"cream\""
        );
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
