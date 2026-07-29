// DeskNote 后端入口
// 阶段 1：仅启动空白窗口，后续阶段逐步添加 commands/events/plugins

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            // 阶段 2-3 将在此初始化 Storage、注册 commands
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
