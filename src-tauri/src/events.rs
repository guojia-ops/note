// DeskNote 事件定义
// PRD 12.6: Rust → 所有窗口 event 通知数据变化
// 前端通过 @tauri-apps/api/event 的 listen 订阅

/// 事件名常量（前后端共享字符串）
pub mod event_name {
    pub const NOTE_CREATED: &str = "note:created";
    pub const NOTE_UPDATED: &str = "note:updated";
    pub const NOTE_DELETED: &str = "note:deleted";
    pub const NOTE_PINNED: &str = "note:pinned";
    pub const NOTE_UNPINNED: &str = "note:unpinned";
    pub const CONFIG_UPDATED: &str = "config:updated";
    /// 托盘/全局快捷键触发「新建便签」→ 主窗口前端监听后调 createNote+pinNote
    pub const TRAY_NEW_NOTE: &str = "tray:new-note";
    /// SOP 10.7：落盘失败通知，前端显示 toast
    pub const STORAGE_ERROR: &str = "storage:error";
}

/// 事件 payload：携带受影响便签的 id
/// 前端收到后按 id 决定如何更新本地 store
#[derive(Debug, Clone, serde::Serialize)]
pub struct NotePayload {
    pub id: String,
}

/// 配置更新事件无 payload（前端整体重新拉取）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfigPayload;
