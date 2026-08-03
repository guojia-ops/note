// DeskNote Tauri Commands
// PRD 12.6: 前端 → Rust command（增删改查），Rust → 所有窗口（event 通知）

use crate::error::{Error, Result};
use crate::events::{event_name, NotePayload};
use crate::state::{AppState, ImportResult, UpdateNoteFields};
use crate::types::{Config, Note, NoteColor};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// SOP 10.7：落盘失败时 emit storage:error 事件，前端显示 toast
/// 命令层调用：state 写操作返回 Err 时，emit 事件后返回错误
fn emit_storage_error(app: &AppHandle, e: &Error) {
    let _ = app.emit(event_name::STORAGE_ERROR, e.to_string());
}

/// 创建便签
/// color 为 None 时使用 config.default_color（PRD 设置项 3）
#[tauri::command]
pub fn create_note(
    app: AppHandle,
    state: State<AppState>,
    title: Option<String>,
    content: Option<String>,
    color: Option<NoteColor>,
) -> Result<Note> {
    let color = color.unwrap_or_else(|| state.get_config().default_color);
    match state.create_note(title.unwrap_or_default(), content.unwrap_or_default(), color) {
        Ok(note) => {
            let _ = app.emit(event_name::NOTE_CREATED, NotePayload { id: note.id.clone() });
            Ok(note)
        }
        Err(e) => {
            emit_storage_error(&app, &e);
            Err(e)
        }
    }
}

/// 更新便签（部分字段）
#[tauri::command]
pub fn update_note(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    fields: UpdateNoteFields,
) -> Result<Note> {
    match state.update_note(&id, &fields) {
        Ok(note) => {
            let _ = app.emit(event_name::NOTE_UPDATED, NotePayload { id: note.id.clone() });
            Ok(note)
        }
        Err(e) => {
            emit_storage_error(&app, &e);
            Err(e)
        }
    }
}

/// 删除便签
#[tauri::command]
pub fn delete_note(app: AppHandle, state: State<AppState>, id: String) -> Result<()> {
    match state.delete_note(&id) {
        Ok(()) => {
            let _ = app.emit(event_name::NOTE_DELETED, NotePayload { id });
            Ok(())
        }
        Err(e) => {
            emit_storage_error(&app, &e);
            Err(e)
        }
    }
}

/// 获取所有便签
#[tauri::command]
pub fn get_notes(state: State<AppState>) -> Result<Vec<Note>> {
    Ok(state.list_notes())
}

/// 贴出便签（仅更新 pinned 状态，窗口由前端 WebviewWindow API 创建）
/// Rust 侧 WebviewUrl::App 对含 query 的路径在 Windows 上解析失败（PathBuf 不支持 ?）
/// 因此窗口创建交给前端，Rust 只负责数据状态，并返回 note 供前端设置初始位置尺寸
#[tauri::command]
pub fn pin_note(app: AppHandle, state: State<AppState>, id: String) -> Result<Note> {
    let note = match state.set_pinned(&id, true) {
        Ok(n) => n,
        Err(e) => {
            emit_storage_error(&app, &e);
            return Err(e);
        }
    };

    // 若窗口已存在则聚焦（前端会创建窗口，这里只处理已存在的情况）
    let label = format!("note-{}", id);
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.show();
        let _ = window.set_focus();
    }

    let _ = app.emit(event_name::NOTE_PINNED, NotePayload { id });
    Ok(note)
}

/// 收回便签（关闭窗口）
#[tauri::command]
pub fn unpin_note(app: AppHandle, state: State<AppState>, id: String) -> Result<()> {
    if let Err(e) = state.set_pinned(&id, false) {
        emit_storage_error(&app, &e);
        return Err(e);
    }

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
    let config = match state.update_config(partial) {
        Ok(c) => c,
        Err(e) => {
            emit_storage_error(&app, &e);
            return Err(e);
        }
    };
    let _ = app.emit(event_name::CONFIG_UPDATED, ());
    Ok(config)
}

/// 导出便签到指定文件路径
#[tauri::command]
pub fn export_notes(state: State<AppState>, path: String) -> Result<()> {
    state.export_notes(&PathBuf::from(path))
}

/// 从指定文件路径导入便签
/// 导入成功后对每条新增便签 emit `note:created` 事件，通知所有窗口刷新
#[tauri::command]
pub fn import_notes(
    app: AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<ImportResult> {
    let result = match state.import_notes(&PathBuf::from(path)) {
        Ok(r) => r,
        Err(e) => {
            emit_storage_error(&app, &e);
            return Err(e);
        }
    };
    // 通知前端刷新：发 imported 数量的 note:created 事件
    // 前端 stores 订阅了 note:created → refreshNotes()，收到即重新拉全量
    if result.imported > 0 {
        let _ = app.emit(event_name::NOTE_CREATED, NotePayload { id: String::new() });
    }
    Ok(result)
}

/// 获取数据目录路径（用于设置页只读显示，PRD 设置项 5）
#[tauri::command]
pub fn get_data_dir(state: State<AppState>) -> Result<String> {
    Ok(state.storage().data_dir().to_string_lossy().to_string())
}

/// 全部收回：遍历 pinned 便签，逐个 set_pinned(false) + 关闭窗口 + emit 事件（SOP 8.11）
#[tauri::command]
pub fn retract_all_notes(app: AppHandle, state: State<AppState>) -> Result<()> {
    let notes = state.list_notes();
    for note in notes.iter().filter(|n| n.pinned) {
        let _ = state.set_pinned(&note.id, false);
        let label = format!("note-{}", note.id);
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.close();
        }
        let _ = app.emit(event_name::NOTE_UNPINNED, NotePayload { id: note.id.clone() });
    }
    Ok(())
}

/// 退出应用（SOP 8.12）
#[tauri::command]
pub fn quit_app(app: AppHandle) -> Result<()> {
    app.exit(0);
    Ok(())
}

/// 重新注册全局快捷键（设置页改快捷键后调用，SOP 8.9）
/// 先 unregister_all 再 register 新 hotkey
#[tauri::command]
pub fn register_hotkey(app: AppHandle, state: State<AppState>) -> Result<()> {
    let hotkey = state.get_config().hotkey;
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| Error::Shortcut(format!("取消旧快捷键失败: {}", e)))?;
    app.global_shortcut()
        .register(hotkey.as_str())
        .map_err(|e| Error::Shortcut(format!("注册快捷键 '{}' 失败: {}", hotkey, e)))?;
    Ok(())
}
