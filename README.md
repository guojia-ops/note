# DeskNote

> 极致轻量的桌面便签应用 —— 便签可贴在屏幕上作为虚拟便利贴，也可在主窗口内管理。
> 采用真实感拟物化设计，让数字便签拥有纸质便利贴的物理质感。

- 安装包体积：**2.8 MB**（NSIS exe）
- 内存占用：空闲 ≤ 30 MB / 典型 ≤ 80 MB / 极端 ≤ 200 MB
- 当前版本：**v1.1.0**（拟物化优化版）

## 功能特性

### 核心功能

- **便签窗口**：无标题栏、透明背景、置顶、不占任务栏，支持拖动 / 调整大小 / 改色 / 编辑，位置与尺寸自动记忆
- **主窗口卡片墙**：网格布局、搜索过滤、新建 / 编辑 / 删除 / 贴出 / 收回
- **设置窗口**：开机自启、全局快捷键、默认颜色、自动备份份数、主题、数据目录、导入导出、关闭行为、便签始终置顶
- **系统托盘**：显示主窗口 / 新建便签 / 全部收回 / 退出
- **全局快捷键**：默认 `Ctrl+Alt+N` 一键新建便签
- **主题**：6 色色板 + `light` / `dark` / `system` 三态，跨窗口实时同步
- **健壮性**：落盘重试、崩溃恢复（从 `backups/` 恢复）、自动备份、`storage:error` 前端提示
- **URL 识别**：正文中的链接自动识别并可交互
- **数据导入导出**：JSON 单文件往返一致

### v1.1 拟物化设计（真实纸质质感）

- **6 色纸质色板**：经典鹅黄 / 樱花粉 / 薄荷绿 / 晴空蓝 / 芋泥紫 / 奶白，低饱和哑光底色模拟纸张观感
- **多层色彩层级**：每色 5 级变量（底色 / 胶条 / 标题文字 / 正文文字 / 卷角背面），区分纸张正面、胶条与反面
- **顶部胶条厚度**：微渐变（顶部受光、底部压痕）+ 1px 顶部亮边 + 1px 底部暗线，模拟胶条压在纸上的物理厚度
- **右下角真实卷角**：径向渐变模拟纸张弯曲曲面，折痕根部深 → 翘起边缘略亮；多层 inset 高光 + 外阴影，纸角像真的翘起离开纸面
- **三层光影结构**：inset 高光（纸张受光面厚度）+ 近景阴影（纸张自身厚度）+ 远景柔阴影（桌面扩散投影）
- **桌面随机微旋转**：每张便签 ±0.8° 随机旋转，模拟随手贴在桌面的真实感
- **拖动抬升反馈**：拖动时阴影放大 + 轻微 scale，模拟拿起便签的物理感
- **进出场动画**：贴出时淡入 + 缩放放大（飞入感），收回时反向缩放淡出
- **暖灰调阴影**：阴影采用棕调暖灰（`rgba(120, 100, 40, ...)`），与米白桌面自然融合，避免纯黑的生硬感

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
│   └── app.css                # CSS 变量主题 + 拟物化色板
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
├── docs/                      # 设计文档
│   └── ui-skeuomorphic-optimization.md
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
| 内存占用（空闲） | ≤ 30 MB | ≤ 30 MB ✓ |
| 内存占用（典型） | ≤ 80 MB | ≤ 80 MB ✓ |

## 拟物化设计要点

### 色板体系（每色 5 级变量）

| 颜色 | 底色 | 胶条色 | 标题文字 | 正文文字 | 卷角背面 |
|------|------|--------|----------|----------|----------|
| 经典鹅黄 | `#FFF4C2` | `#F0D97D` | `#5A4A1F` | `#6B5A2B` | `#E8D27A` |
| 樱花粉 | `#FFE6E6` | `#F5C2C2` | `#5C2B2B` | `#703A3A` | `#EFC0C0` |
| 薄荷绿 | `#E8F5E0` | `#BED9A8` | `#2F4A23` | `#3E5C30` | `#B8D4A3` |
| 晴空蓝 | `#E3F0FC` | `#A9CBEE` | `#1F3D5C` | `#2C4E73` | `#A0C4E8` |
| 芋泥紫 | `#F1E8F7` | `#D4BEE4` | `#432C5A` | `#543B6E` | `#CCB5DD` |
| 奶白 | `#FBF9F3` | `#DDD4BF` | `#3A3A3A` | `#4F4F4F` | `#D5CDB8` |

### 阴影分层（v2.5）

```css
/* 便签三层光影结构 */
box-shadow:
  inset 0 1px 0 rgba(255, 255, 255, 0.35),    /* 顶部受光高光 */
  inset 1px 0 0 rgba(255, 255, 255, 0.15),    /* 左侧受光高光 */
  0 1px 2px rgba(120, 100, 40, 0.08),         /* 近景：纸张厚度 */
  0 6px 16px rgba(120, 100, 40, 0.10),        /* 中景：贴身投影 */
  0 14px 32px rgba(120, 100, 40, 0.12);       /* 远景：桌面扩散 */
```

### 右下角卷角（v2.5.2）

- 尺寸：40×32（不等腰，向左上翘起更多）
- 径向渐变背面：折痕根部 `fold 35% + 黑` → 翘起边缘 `fold 75% + 白`
- inset 高光：顶部左侧 1px 亮边（折痕受光）+ 内侧暗角（背面凹陷）
- 外阴影：多层叠加，翘起纸角投在便签正面的渐变阴影
- 桌面投影：54×46 径向阴影，向左下扩散

## 文档

- [PRD.md](./PRD.md) —— 产品需求文档
- [SOP.md](./SOP.md) —— 开发标准操作流程
- [docs/ui-skeuomorphic-optimization.md](./docs/ui-skeuomorphic-optimization.md) —— 拟物化 UI 优化方案

## License

Copyright © 2026 DeskNote
