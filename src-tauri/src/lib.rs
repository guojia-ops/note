// DeskNote 后端入口
// 阶段 8：托盘 + 全局快捷键 + 主窗口关闭行为

#![allow(dead_code)]

mod commands;
mod error;
mod events;
mod state;
mod storage;
mod types;

use events::{event_name, NotePayload};
use state::AppState;
use storage::Storage;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_autostart::ManagerExt;
use types::CloseAction;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // SOP 8.6-8.8：全局快捷键插件，handler 在按下时 emit 新建便签事件
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let _ = app.emit(event_name::TRAY_NEW_NOTE, ());
                    }
                })
                .build(),
        )
        // 开机自启插件（v1.1 优化阶段 3）
        // MacosLauncher::LaunchAgent 用于 macOS，Windows/Linux 自动选择注册表/桌面文件
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // SOP 8.10：主窗口关闭行为拦截
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    let state = window.app_handle().state::<AppState>();
                    let config = state.get_config();
                    match config.close_action {
                        CloseAction::Tray => {
                            let _ = window.hide();
                            api.prevent_close();
                        }
                        CloseAction::Quit => {
                            window.app_handle().exit(0);
                        }
                    }
                }
            }
        })
        .setup(|app| {
            // 解析 Tauri appDataDir（作为 D 盘不存在时的回退路径）
            let app_data_dir = app.path().app_data_dir().ok();

            // 初始化 Storage
            let state = match Storage::new(app_data_dir.as_deref()) {
                Ok(storage) => {
                    let data_dir = storage.data_dir().to_string_lossy().to_string();
                    println!("[DeskNote] 数据目录: {}", data_dir);

                    match AppState::from_storage(storage) {
                        Ok(state) => {
                            let note_count = state.list_notes().len();
                            println!("[DeskNote] 加载便签 {} 条", note_count);
                            state
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
            };

            // SOP 8.7：启动时从 config 读取快捷键并注册
            let hotkey = state.get_config().hotkey.clone();
            // v1.1 优化阶段 3：启动时同步开机自启状态
            // config.auto_start 为 SOT，但用户可能在系统设置中手动改过，这里以 config 为准强制同步
            let auto_start = state.get_config().auto_start;

            app.manage(state);
            if let Err(e) = app.global_shortcut().register(hotkey.as_str()) {
                eprintln!("[DeskNote] 全局快捷键注册失败 '{}': {}", hotkey, e);
            } else {
                println!("[DeskNote] 全局快捷键已注册: {}", hotkey);
            }

            // 同步开机自启：config 开启则 enable，关闭则 disable
            let autolaunch = app.autolaunch();
            let current_enabled = autolaunch.is_enabled().unwrap_or(false);
            if auto_start && !current_enabled {
                if let Err(e) = autolaunch.enable() {
                    eprintln!("[DeskNote] 开机自启启用失败: {}", e);
                }
            } else if !auto_start && current_enabled {
                if let Err(e) = autolaunch.disable() {
                    eprintln!("[DeskNote] 开机自启关闭失败: {}", e);
                }
            }

            // SOP 8.2-8.5：托盘图标 + 四项菜单
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let new_item = MenuItem::with_id(app, "new", "新建便签", true, None::<&str>)?;
            let retract_item = MenuItem::with_id(app, "retract", "全部收回", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &new_item, &retract_item, &quit_item])?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("DeskNote")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "new" => {
                        // 通知主窗口前端调 createNote + pinNote
                        let _ = app.emit(event_name::TRAY_NEW_NOTE, ());
                    }
                    "retract" => {
                        // SOP 8.11：全部收回
                        let state = app.state::<AppState>();
                        let notes = state.list_notes();
                        for note in notes.iter().filter(|n| n.pinned) {
                            let _ = state.set_pinned(&note.id, false);
                            let label = format!("note-{}", note.id);
                            if let Some(window) = app.get_webview_window(&label) {
                                let _ = window.close();
                            }
                            let _ =
                                app.emit(event_name::NOTE_UNPINNED, NotePayload { id: note.id.clone() });
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // SOP 8.4：左键单击 toggle 主窗口显示/隐藏
                    if let TrayIconEvent::Click { button, button_state, .. } = event {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::get_notes,
            commands::pin_note,
            commands::unpin_note,
            commands::complete_note,
            commands::uncomplete_note,
            commands::get_config,
            commands::update_config,
            commands::export_notes,
            commands::import_notes,
            commands::get_data_dir,
            commands::retract_all_notes,
            commands::quit_app,
            commands::register_hotkey,
            commands::set_auto_start,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
