// DeskNote Tauri Commands
// PRD 12.6: 前端 → Rust command（增删改查），Rust → 所有窗口（event 通知）

use crate::error::Result;
use crate::events::{event_name, NotePayload};
use crate::state::{AppState, ImportResult, UpdateNoteFields};
use crate::types::{Config, Note, NoteColor};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindowBuilder};

/// 创建便签
#[tauri::command]
pub fn create_note(
    app: AppHandle,
    state: State<AppState>,
    title: Option<String>,
    content: Option<String>,
    color: Option<NoteColor>,
) -> Result<Note> {
    let note = state.create_note(
        title.unwrap_or_default(),
        content.unwrap_or_default(),
        color.unwrap_or_default(),
    );
    let _ = app.emit(event_name::NOTE_CREATED, NotePayload { id: note.id.clone() });
    Ok(note)
}

/// 更新便签（部分字段）
#[tauri::command]
pub fn update_note(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    fields: UpdateNoteFields,
) -> Result<Note> {
    let note = state.update_note(&id, &fields)?;
    let _ = app.emit(event_name::NOTE_UPDATED, NotePayload { id: note.id.clone() });
    Ok(note)
}

/// 删除便签
#[tauri::command]
pub fn delete_note(app: AppHandle, state: State<AppState>, id: String) -> Result<()> {
    state.delete_note(&id)?;
    let _ = app.emit(event_name::NOTE_DELETED, NotePayload { id });
    Ok(())
}

/// 获取所有便签
#[tauri::command]
pub fn get_notes(state: State<AppState>) -> Result<Vec<Note>> {
    Ok(state.list_notes())
}

/// 贴出便签（创建独立窗口）
#[tauri::command]
pub fn pin_note(app: AppHandle, state: State<AppState>, id: String) -> Result<()> {
    let note = state.set_pinned(&id, true)?;

    let label = format!("note-{}", id);
    let url = format!("note.html?id={}", id);

    // 若窗口已存在则聚焦，否则创建
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::App(url.into()))
            .title("DeskNote")
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(true)
            .inner_size(note.width as f64, note.height as f64)
            .position(note.x as f64, note.y as f64)
            .build()?;
    }

    let _ = app.emit(event_name::NOTE_PINNED, NotePayload { id });
    Ok(())
}

/// 收回便签（关闭窗口）
#[tauri::command]
pub fn unpin_note(app: AppHandle, state: State<AppState>, id: String) -> Result<()> {
    state.set_pinned(&id, false)?;

    let label = format!("note-{}", id);
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.close();
    }

    let _ = app.emit(event_name::NOTE_UNPINNED, NotePayload { id });
    Ok(())
}

/// 获取配置
#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<Config> {
    Ok(state.get_config())
}

/// 更新配置（部分字段，传 JSON 对象）
#[tauri::command]
pub fn update_config(
    app: AppHandle,
    state: State<AppState>,
    partial: serde_json::Value,
) -> Result<Config> {
    let config = state.update_config(partial)?;
    let _ = app.emit(event_name::CONFIG_UPDATED, ());
    Ok(config)
}

/// 导出便签到指定文件路径
#[tauri::command]
pub fn export_notes(state: State<AppState>, path: String) -> Result<()> {
    state.export_notes(&PathBuf::from(path))
}

/// 从指定文件路径导入便签
#[tauri::command]
pub fn import_notes(state: State<AppState>, path: String) -> Result<ImportResult> {
    state.import_notes(&PathBuf::from(path))
}

/// 获取数据目录路径（用于设置页只读显示，PRD 设置项 5）
#[tauri::command]
pub fn get_data_dir(state: State<AppState>) -> Result<String> {
    Ok(state.storage().data_dir().to_string_lossy().to_string())
}
