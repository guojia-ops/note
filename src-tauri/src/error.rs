// DeskNote 错误类型
// 统一的 error::Error，便于 Tauri command 跨边界传递

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("路径解析错误: {0}")]
    Path(String),

    #[error("便签不存在: {0}")]
    NotFound(String),

    #[error("Tauri 错误: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("全局快捷键错误: {0}")]
    Shortcut(String),
}

// 为 Tauri command 跨边界传递实现 Serialize
// 用字符串形式传递错误信息，前端收到 { message: "..." }
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
