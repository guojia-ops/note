// DeskNote 后端入口
// 阶段 2：数据层（types / storage / error）已就绪，setup 阶段初始化 Storage

// 阶段 2 临时：数据层 API 将在阶段 3 commands 中被调用，此前会有 dead_code 告警
// 阶段 3 完成后移除此 allow
#![allow(dead_code)]

mod error;
mod storage;
mod types;

use storage::Storage;
use tauri::Manager; // app_handle.path()

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 解析 Tauri appDataDir（作为 D 盘不存在时的回退路径）
            let app_data_dir = app
                .path()
                .app_data_dir()
                .ok();

            // 初始化 Storage（解析数据目录、创建目录结构）
            match Storage::new(app_data_dir.as_deref()) {
                Ok(storage) => {
                    let data_dir = storage.data_dir().to_string_lossy().to_string();
                    println!("[DeskNote] 数据目录: {}", data_dir);

                    // 尝试加载数据验证可读性
                    match storage.load_data() {
                        Ok(data) => println!(
                            "[DeskNote] 加载便签 {} 条",
                            data.notes.len()
                        ),
                        Err(e) => eprintln!("[DeskNote] 数据加载失败: {}", e),
                    }
                    match storage.load_config() {
                        Ok(config) => println!("[DeskNote] 配置加载成功: hotkey={}", config.hotkey),
                        Err(e) => eprintln!("[DeskNote] 配置加载失败: {}", e),
                    }

                    // 存入应用状态（阶段 3 将通过 State 访问）
                    app.manage(storage);
                }
                Err(e) => {
                    eprintln!("[DeskNote] Storage 初始化失败: {}", e);
                    return Err(e.into());
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
