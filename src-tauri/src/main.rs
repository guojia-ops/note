// Tauri 2.x 入口：调用 lib 的 run 函数
// 防止 Windows release 构建 Console 窗口弹出
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    desknote_lib::run()
}
