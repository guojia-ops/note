// DeskNote 后端入口
// 阶段 3：通信层（commands + events + state）已就绪

// 阶段 3 临时：以下 API 将在后续阶段被调用
// - storage: get_note / backup_dir / create_backup / cleanup_old_backups → 阶段 8 托盘、阶段 10 健壮性
// - types: NoteColor::as_css_var → 阶段 5 便签窗口（实际前端用 CSS 变量映射，此方法可能删）
// 上述阶段完成后移除此 allow
#![allow(dead_code)]

mod commands;
mod error;
mod events;
mod state;
mod storage;
mod types;

use state::AppState;
use storage::Storage;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 解析 Tauri appDataDir（作为 D 盘不存在时的回退路径）
            let app_data_dir = app.path().app_data_dir().ok();

            // 初始化 Storage
            match Storage::new(app_data_dir.as_deref()) {
                Ok(storage) => {
                    let data_dir = storage.data_dir().to_string_lossy().to_string();
                    println!("[DeskNote] 数据目录: {}", data_dir);

                    // 初始化 AppState（加载已有数据）
                    match AppState::from_storage(storage) {
                        Ok(state) => {
                            let note_count = state.list_notes().len();
                            println!("[DeskNote] 加载便签 {} 条", note_count);
                            app.manage(state);
                        }
                        Err(e) => {
                            eprintln!("[DeskNote] AppState 初始化失败: {}", e);
                            return Err(e.into());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[DeskNote] Storage 初始化失败: {}", e);
                    return Err(e.into());
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::get_notes,
            commands::pin_note,
            commands::unpin_note,
            commands::get_config,
            commands::update_config,
            commands::export_notes,
            commands::import_notes,
            commands::get_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
