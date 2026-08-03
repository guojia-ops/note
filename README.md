# DeskNote

> 极致轻量的桌面便签应用 —— 便签可贴在屏幕上作为虚拟便利贴，也可在主窗口内管理。

- 安装包体积：**2.8 MB**（NSIS exe）
- 内存占用：空闲 ≤ 30 MB / 典型 ≤ 80 MB / 极端 ≤ 200 MB
- 当前版本：**v1.0.0**（MVP）

## 功能特性

- **便签窗口**：无标题栏、透明背景、置顶、不占任务栏，支持拖动 / 调整大小 / 改色 / 编辑，位置与尺寸自动记忆
- **主窗口卡片墙**：网格布局、搜索过滤、新建 / 编辑 / 删除 / 贴出 / 收回
- **设置窗口**：开机自启、全局快捷键、默认颜色、自动备份份数、主题、数据目录、导入导出、关闭行为
- **系统托盘**：显示主窗口 / 新建便签 / 全部收回 / 退出
- **全局快捷键**：默认 `Ctrl+Alt+N` 一键新建便签
- **主题**：6 色色板 + `light` / `dark` / `system` 三态，跨窗口实时同步
- **健壮性**：落盘重试、崩溃恢复（从 `backups/` 恢复）、自动备份、`storage:error` 前端提示
- **URL 识别**：正文中的链接自动识别并可交互
- **数据导入导出**：JSON 单文件往返一致

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面运行时 | Tauri 2.x（Rust 壳 + 系统 WebView2） |
| 前端框架 | Svelte 5 |
| 语言 | TypeScript（strict）/ Rust |
| 构建 | Vite |
| 包管理 | pnpm |
| 状态管理 | Svelte 内置 stores（零依赖） |
| 测试 | Vitest（前端）+ `cargo test`（Rust） |
| 打包 | Tauri bundle（NSIS） |

前端 dependencies ≤ 10 个，无状态管理库、无 UI 组件库。

## 快速开始

### 环境要求

- Node.js ≥ 20.x
- pnpm ≥ 9.x
- Rust ≥ 1.75
- Windows：Visual Studio Build Tools（C++ 工作负载）+ WebView2 Runtime

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri dev
```

> 在 TRAE IDE 内运行时，需在外部 PowerShell 中执行 `.\node_modules\.bin\tauri dev` 以绕过 AppData 写入限制。

### 打包

```bash
pnpm tauri build
```

产物位于 `src-tauri/target/release/bundle/nsis/*.exe`。

### 测试

```bash
# 前端单元测试
pnpm test

# Rust 单元测试
cd src-tauri && cargo test
```

### 类型检查

```bash
pnpm check
```

## 项目结构

```
note/
├── src/                       # 前端
│   ├── main.ts / note.ts / settings.ts   # 三窗口入口
│   ├── views/                 # MainApp / NoteApp / Settings
│   ├── components/            # NoteCard / NoteEditModal
│   ├── stores/                # notes / config（Svelte stores）
│   ├── lib/                   # commands / events / url / time
│   ├── types/                 # 与 Rust 对齐的类型
│   └── app.css                # CSS 变量主题
├── src-tauri/                 # Rust 壳
│   ├── src/
│   │   ├── lib.rs             # setup / invoke_handler / 托盘 / 快捷键
│   │   ├── commands.rs        # Tauri commands
│   │   ├── state.rs           # AppState（Mutex）
│   │   ├── storage.rs         # JSON 持久化 + 备份
│   │   ├── types.rs           # Note / Config / 枚举
│   │   ├── events.rs          # 事件常量
│   │   └── error.rs           # 统一错误
│   ├── capabilities/          # Tauri 2.x 权限
│   └── tauri.conf.json
├── PRD.md                     # 产品需求文档
├── SOP.md                     # 开发标准操作流程（13 阶段）
└── package.json
```

## 数据存储

- **主路径**：`D:\ProgramData\notes_data\`（D 盘存在时）
- **回退路径**：Tauri `appDataDir()`（D 盘不存在时）
- **文件**：
  - `data.json` —— 便签数据（Rust 内存状态为 SOT，防抖落盘）
  - `config.json` —— 应用配置
  - `backups/data_<timestamp>.json` —— 自动备份（按 retention 滚动清理）

## 通信机制

- **Commands**（前端 → Rust）：`create_note` / `update_note` / `delete_note` / `pin_note` / `unpin_note` / `get_config` / `update_config` / `export_notes` / `import_notes`
- **Events**（Rust → 前端广播）：`note:created` / `note:updated` / `note:deleted` / `note:pinned` / `note:unpinned` / `config:updated` / `storage:error`

## 验收指标

| 项 | 目标 | 实测 |
|---|---|---|
| 安装包体积 | ≤ 10 MB | 2.8 MB ✓ |
| 类型检查 | 0 errors | 0 errors ✓ |
| Rust 单元测试 | 全通过 | 24 tests ✓ |
| 前端单元测试 | 全通过 | 28 tests ✓ |

## 文档

- [PRD.md](./PRD.md) —— 产品需求文档
- [SOP.md](./SOP.md) —— 开发标准操作流程

## License

Copyright © 2026 DeskNote
