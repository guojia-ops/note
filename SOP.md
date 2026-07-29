# DeskNote 开发 SOP（标准操作流程）

> **配套文档**：[PRD.md](./PRD.md)
> **版本**：v1.0 MVP
> **日期**：2026-07-27
> **粒度**：中粒度任务清单（阶段 → 任务 → 验收点）
> **目标**：从零到 MVP 交付的可执行开发流程

---

## 0. 总览

### 0.1 阶段流程图

```
[阶段 0 环境准备]
       ↓
[阶段 1 项目初始化]
       ↓
[阶段 2 数据层 Rust] ──┐
       ↓                │
[阶段 3 通信层 Rust] ◄──┘
       ↓
[阶段 4 前端基础设施]
       ↓
   ┌───┴───┬────────┬────────┐
   ↓       ↓        ↓        ↓
[阶段5   [阶段6   [阶段7   [阶段8
便签窗口] 主窗口]  设置窗口] 托盘快捷键]
   └───┬───┴────────┴────────┘
       ↓
[阶段 9 主题视觉]
       ↓
[阶段 10 健壮性备份]
       ↓
[阶段 11 测试]
       ↓
[阶段 12 打包发布]
       ↓
[阶段 13 MVP 验收]
```

### 0.2 阶段依赖关系

| 阶段 | 前置依赖 | 是否阻塞后续 |
|---|---|---|
| 0 环境准备 | 无 | 是 |
| 1 项目初始化 | 0 | 是 |
| 2 数据层 | 1 | 是 |
| 3 通信层 | 2 | 是 |
| 4 前端基础设施 | 3 | 是 |
| 5 便签窗口 | 4 | 否（可与 6/7/8 并行） |
| 6 主窗口 | 4 | 否 |
| 7 设置窗口 | 4 | 否 |
| 8 托盘快捷键 | 4 | 否 |
| 9 主题视觉 | 5,6,7 | 否 |
| 10 健壮性 | 2,3 | 否 |
| 11 测试 | 5-8 | 是 |
| 12 打包 | 11 | 是 |
| 13 验收 | 12 | 是 |

### 0.3 全局约束

- **不引入 PRD 未列出的依赖**（前端 dependencies ≤ 10 个）
- **每个任务的验收点必须通过才能进入下一阶段**（阻塞型阶段）
- **代码提交粒度 = 任务粒度**，每个任务一次 commit
- **TS 严格模式**：`strict: true`，禁用 `any`
- **Rust**：`clippy` 无 warning，`rustfmt` 默认配置

---

## 阶段 0：环境准备

### 目标
确保开发机器具备所有工具链。

### 任务清单

- [x] **0.1** 安装 Node.js ≥ 20.x（LTS）—— 实测 v22.16.0
- [x] **0.2** 启用 corepack：`corepack enable`
- [x] **0.3** 安装 pnpm ≥ 9.x：`corepack prepare pnpm@latest --activate` —— 实测 11.17.0
- [x] **0.4** 执行 `pnpm setup` 配置 pnpm 全局 bin 目录到 PATH（重启终端生效）
- [x] **0.5** 安装 Rust 工具链（rustup）：`https://rustup.rs` —— 实测 1.97.1
- [x] **0.6** 验证 Rust：`rustc --version` ≥ 1.75
- [x] **0.7** Windows：安装 Visual Studio Build Tools（含 C++ 工作负载）或完整 VS —— 实测 VS 生成工具 2026
- [x] **0.8** Windows：确认 WebView2 Runtime 已安装（Win11 默认有，Win10 早期版本需手动装）—— 实测 150.0.4078.99
- [x] **0.9** 安装 Tauri 2.x CLI（pnpm 全局）：`pnpm add -g @tauri-apps/cli@^2` —— 实测 2.11.4
- [x] **0.10** 验证 Tauri 前置条件：`tauri info` 无错误

### 验收点
- `node -v`、`pnpm -v`、`rustc --version`、`cargo --version`、`tauri --version` 全部正常输出
- `tauri info` 不报缺失依赖

---

## 阶段 1：项目初始化

### 目标
搭建 Tauri + Svelte + Vite + TS 项目骨架，可启动空白窗口。

### 任务清单

- [ ] **1.1** 用 create-tauri-app 脚手架初始化：
  ```
  pnpm create tauri-app desknote
  ```
  选 Svelte + TypeScript + pnpm

- [ ] **1.2** 配置 `tauri.conf.json`：
  - `identifier`: `com.notes.sticky`
  - `productName`: `DeskNote`
  - `app.windows[0].title`: `DeskNote`
  - `app.windows[0].label`: `main`
  - 默认尺寸 1000 × 700

- [ ] **1.3** 配置 `package.json`：
  - `name`: `desknote`
  - 添加 `type: "module"`
  - 配置 scripts：`dev`、`build`、`tauri:dev`、`tauri:build`

- [ ] **1.4** 配置 `tsconfig.json`：开启 `strict`、`noUnusedLocals`、`noImplicitAny`

- [ ] **1.5** 配置 Vite 多入口（主窗口 + 便签窗口）：
  - `build.rollupOptions.input.main`
  - `build.rollupOptions.input.note`

- [ ] **1.6** 创建目录骨架（PRD 4.4 节）：
  ```
  src/
  ├── main.ts
  ├── note.ts
  ├── views/
  ├── stores/
  ├── lib/
  └── types/
  ```

- [ ] **1.7** 配置 `.gitignore`（node_modules、target、dist、.DS_Store）
- [ ] **1.8** 配置 `.editorconfig` + Prettier + ESLint（Svelte + TS 规则）
- [ ] **1.9** 初始化 git：`git init` + 首次 commit `chore: init project`

### 验收点
- `pnpm tauri dev` 能启动主窗口
- 浏览器控制台无报错
- `cargo tauri info` 无警告

---

## 阶段 2：数据层（Rust）

### 目标
实现 Note / Config 数据模型、内存状态、JSON 持久化、路径解析。

### 任务清单

- [ ] **2.1** 在 `src-tauri/src/types.rs` 定义 Note / NoteColor / Config 结构体（与 PRD 3.1 / 3.2 对齐），派生 `Serialize`/`Deserialize`/`Clone`

- [ ] **2.2** 实现 `NoteColor` 6 色枚举：`yellow/pink/green/blue/purple/orange`

- [ ] **2.3** 实现 `Storage` 模块（`src-tauri/src/storage.rs`）：
  - `Storage::resolve_root()`：D 盘存在 → `D:\ProgramData\notes_data\`，否则 → `appDataDir()`
  - `Storage::ensure_dirs()`：创建 root + `backups/`
  - `Storage::load_notes() -> Vec<Note>`：读 `data.json`，损坏返回空 + 日志
  - `Storage::load_config() -> Config`：读 `config.json`，缺失返回默认
  - `Storage::save_notes(&[Note])`：原子写（写 `.tmp` 后 rename）
  - `Storage::save_config(&Config)`
  - `Storage::backup()`：复制 `data.json` 到 `backups/data_<timestamp>.json`，超出 retention 删最旧

- [ ] **2.4** 实现路径解析的单元测试（mock D 盘存在/不存在两种情况）

- [ ] **2.5** 实现默认 Config：
  ```rust
  Config {
      auto_start: false,
      global_hotkey: "Ctrl+Alt+N".into(),
      default_color: NoteColor::Yellow,
      auto_backup: true,
      backup_retention: 5,
      theme: Theme::System,
      close_behavior: CloseBehavior::Tray,
  }
  ```

- [ ] **2.6** 在 `lib.rs` `setup` 钩子里初始化 Storage，加载 notes + config 到 `State<Mutex<AppState>>`

- [ ] **2.7** 实现防抖落盘：用 `tokio::spawn` + `sleep` + `AtomicBool` 标记 dirty，500ms 内合并写入

### 验收点
- `cargo test` 通过
- 启动应用后 `D:\ProgramData\notes_data\` 自动创建（含 `backups/`）
- 手动塞数据到 `data.json`，重启应用能读到（通过日志验证）

---

## 阶段 3：通信层（Rust Commands + Events）

### 目标
暴露 Tauri commands 给前端，实现事件广播。

### 任务清单

- [ ] **3.1** 定义 commands（`src-tauri/src/commands.rs`）：
  - `create_note(title, content, color) -> Note`
  - `update_note(id, fields) -> Note`
  - `delete_note(id) -> ()`
  - `get_notes() -> Vec<Note>`
  - `pin_note(id) -> ()`（创建便签窗口）
  - `unpin_note(id) -> ()`（关闭便签窗口）
  - `get_config() -> Config`
  - `update_config(partial) -> Config`
  - `export_notes(path) -> ()`
  - `import_notes(path) -> ImportResult`

- [ ] **3.2** 在 `lib.rs` `invoke_handler![]` 注册所有 commands

- [ ] **3.3** 实现事件广播：每个写操作完成后 `app.emit("note:updated", payload)`，前端订阅刷新

- [ ] **3.4** 定义事件类型常量（前端/后端共享）：
  - `note:created` / `note:updated` / `note:deleted` / `note:pinned` / `note:unpinned`
  - `config:updated`

- [ ] **3.5** `pin_note` 实现：调用 `WebviewWindowBuilder` 创建便签窗口
  - label: `note-<id>`
  - url: `note.html?id=<id>`（前端按 query 取数据）
  - decorations: false / transparent: true / alwaysOnTop: true / skipTaskbar: true
  - 位置/尺寸从 Note 字段恢复

- [ ] **3.6** `unpin_note` 实现：通过 label 关闭窗口

- [ ] **3.7** Rust 侧单元测试：commands 业务逻辑（mock State）

### 验收点
- 前端 `invoke('create_note', ...)` 能创建并返回 Note
- 创建后能收到 `note:created` 事件
- `data.json` 持久化生效
- 便签窗口能被 `pin_note` 创建出来（即使是空白也行）

---

## 阶段 4：前端基础设施

### 目标
搭建前端共享层：类型、commands 封装、events 订阅、stores。

### 任务清单

- [ ] **4.1** `src/types/note.ts`：定义 Note / NoteColor / Config 类型，与 Rust 侧对齐
- [ ] **4.2** `src/lib/commands.ts`：封装所有 `invoke` 调用，返回 Promise<T>，类型安全
- [ ] **4.3** `src/lib/events.ts`：封装 `listen` 订阅，导出 `onNoteCreated(cb)` 等辅助函数
- [ ] **4.4** `src/stores/notes.ts`：`writable<Note[]>` + `refresh()` + `subscribe(events)`
- [ ] **4.5** `src/stores/config.ts`：`writable<Config>` + `update(partial)` + 订阅 `config:updated`
- [ ] **4.6** `src/lib/url.ts`：URL 识别正则 + 渲染辅助
- [ ] **4.7** 全局样式 `src/app.css`：CSS 变量定义主题色（PRD 6.1 / 6.4 节色值）
- [ ] **4.8** 在 `main.ts` 启动时调用 `refresh()` 拉取数据

### 验收点
- 主窗口能渲染 notes store 中的便签数量（即使是 0）
- 手动 `invoke('create_note')` 后，store 自动更新（事件订阅生效）

---

## 阶段 5：便签窗口

### 目标
实现完整的便签窗口 UI 与交互。

### 任务清单

- [ ] **5.1** `src/note.ts`：便签窗口入口，从 URL query 解析 `id`，加载对应 Note
- [ ] **5.2** `src/views/Note.svelte`：便签窗口主组件
- [ ] **5.3** 顶部把手（高 32px）：
  - `data-tauri-drag-region` 启用拖动
  - 右侧关闭按钮 ×，点击调用 `unpin_note(id)` 关闭窗口
- [ ] **5.4** 正文区 `<textarea>`：
  - 绑定 `content`，`on:input` 防抖 500ms 调 `update_note`
  - URL 识别：渲染时高亮链接（textarea 内不渲染，仅在 hover tooltip 显示可点击）
  - 注：textarea 内无法真正渲染超链接，需配合 overlay 层或仅在「只读模式」渲染链接。MVP 简化：textarea + 底部状态栏提示「N 个链接」
- [ ] **5.5** 底部工具条（高 36px）：
  - 6 色色块（20×20px），点击调 `update_note(id, {color})`
  - 当前色高亮（边框）
- [ ] **5.6** 调整大小：Tauri 原生 `resizable: true`，监听 `tauri://resize` 事件更新 `width/height`
- [ ] **5.7** 拖动结束：监听 `tauri://move` 事件更新 `x/y`（防抖）
- [ ] **5.8** 窗口背景：透明 + 圆角矩形 + 阴影（PRD 6.2 节）
- [ ] **5.9** 标题栏：标题输入框（可空），失焦保存
- [ ] **5.10** 订阅 `note:updated` 事件，若 id 匹配则同步本地状态（多窗口编辑一致性）

### 验收点
- 便签窗口无标题栏、透明背景、置顶、不在任务栏
- 拖动、调整大小、改色、编辑、关闭全部可用
- 关闭后 `pinned = false`，主窗口列表状态更新
- 重启应用，再次贴出恢复上次位置和尺寸

---

## 阶段 6：主窗口

### 目标
实现主窗口网格卡片墙及全部管理功能。

### 任务清单

- [ ] **6.1** `src/views/Main.svelte`：主窗口根组件
- [ ] **6.2** 顶部栏：
  - 应用名「DeskNote」
  - 搜索框：`on:input` 防抖 200ms，过滤 `title + content`
  - 「+ 新建」按钮：调 `create_note` + `pin_note`
  - 「⚙ 设置」按钮：打开设置窗口
- [ ] **6.3** 网格卡片墙：
  - CSS Grid，`grid-template-columns: repeat(auto-fill, minmax(200px, 1fr))`
  - 卡片宽高比 4:3
- [ ] **6.4** 卡片组件 `NoteCard.svelte`：
  - 顶部色条（便签颜色）
  - 标题（空则显示正文前 30 字）
  - 正文预览（截断 2 行）
  - 更新时间（相对时间，如「3 分钟前」）
- [ ] **6.5** 卡片交互：
  - 双击 → 打开模态编辑面板（同窗口内浮层）
  - 右键菜单：贴出/收回、编辑、删除
  - 悬浮按钮：删除（带确认弹窗）
- [ ] **6.6** 贴出/收回按钮：
  - `pinned = false` → 显示「贴出」按钮，调 `pin_note(id)`
  - `pinned = true` → 显示「收回」按钮，调 `unpin_note(id)`
- [ ] **6.7** 删除确认弹窗：原生 `confirm` 或自绘 modal
- [ ] **6.8** 编辑模态：标题输入 + 正文 textarea + 保存/取消
- [ ] **6.9** 空状态：无便签时显示「点击 + 新建创建第一张便签」
- [ ] **6.10** 订阅 `note:created/updated/deleted/pinned/unpinned` 事件刷新列表

### 验收点
- 创建便签后卡片墙立即出现新卡片
- 贴出/收回状态在卡片上正确显示
- 搜索过滤实时生效
- 删除有确认，删除后卡片消失
- 双击编辑可改标题和正文

---

## 阶段 7：设置窗口

### 目标
实现设置窗口 UI 与全部 8 项设置。

### 任务清单

- [ ] **7.1** 在 `tauri.conf.json` 预声明 `settings` 窗口（或运行时动态创建）
- [ ] **7.2** `src/settings.ts`：设置窗口入口
- [ ] **7.3** `src/views/Settings.svelte`：设置表单
- [ ] **7.4** 开机自启开关：调用 `tauri-plugin-autostart`
- [ ] **7.5** 全局快捷键输入：调用 `tauri-plugin-global-shortcut`，注册/取消注册
- [ ] **7.6** 默认颜色色板选择
- [ ] **7.7** 数据目录只读显示：调 Rust command 获取当前实际路径
- [ ] **7.8** 自动备份开关 + 份数输入（数字 1-20）
- [ ] **7.9** 主题三选一：`light/dark/system`，立即应用 CSS 变量
- [ ] **7.10** 导出按钮：调 `dialog.save()` 选位置 → `export_notes(path)`
- [ ] **7.11** 导入按钮：调 `dialog.open()` 选文件 → `import_notes(path)` → 显示导入结果（新增/跳过数）
- [ ] **7.12** 关闭主窗口行为单选：`tray/quit`
- [ ] **7.13** 每个设置项变更立即调 `update_config(partial)` 持久化

### 验收点
- 所有 8 项设置可读可写
- 配置变更持久化到 `config.json`
- 主题切换立即生效
- 全局快捷键变更后旧快捷键失效、新快捷键生效
- 导出/导入往返一致（导出后清空再导入，数据一致）

---

## 阶段 8：托盘与全局快捷键

### 目标
实现系统托盘和全局快捷键。

### 任务清单

- [ ] **8.1** 安装 `tauri-plugin-system-tray`（Tauri 2.x 内置或插件）
- [ ] **8.2** 配置托盘图标（复用应用图标，待图标生成后放入 `src-tauri/icons/`）
- [ ] **8.3** 托盘菜单四项：
  - 显示主窗口
  - 新建便签
  - 全部收回
  - 退出
- [ ] **8.4** 左键单击：toggle 主窗口显示/隐藏
- [ ] **8.5** 右键单击：弹出菜单
- [ ] **8.6** 安装 `tauri-plugin-global-shortcut`
- [ ] **8.7** 启动时从 config 读取快捷键并注册
- [ ] **8.8** 快捷键触发：调 `create_note` + `pin_note`
- [ ] **8.9** 设置项变更快捷键时，先取消旧注册再注册新的
- [ ] **8.10** 主窗口关闭行为：
  - `close_behavior = tray`：拦截 close 事件，hide 主窗口
  - `close_behavior = quit`：调用 `app.exit(0)`
- [ ] **8.11** 「全部收回」：遍历 `pinned = true` 的便签，逐个 `unpin_note`
- [ ] **8.12** 「退出」：先保存所有 dirty 数据，再 exit

### 验收点
- 托盘图标可见，菜单四项可用
- 全局快捷键在任何应用聚焦时都能触发新建
- 关闭主窗口后托盘仍在，便签仍贴在桌面
- 「全部收回」一键关闭所有便签窗口

---

## 阶段 9：主题与视觉打磨

### 目标
落地视觉规范，确保跨主题一致性。

### 任务清单

- [ ] **9.1** 落地 6 色板色值（PRD 6.1 节）到 CSS 变量
- [ ] **9.2** 实现主题切换：
  - `light` / `dark` / `system` 三态
  - `system` 监听 `prefers-color-scheme` 媒体查询
- [ ] **9.3** 便签窗口视觉细节（PRD 6.2 节）：
  - 圆角 8px
  - 阴影 `box-shadow: 0 4px 12px rgba(0,0,0,0.15)`
  - 顶部把手 hover 拖动光标
  - 关闭按钮 hover 变红
- [ ] **9.4** 主窗口卡片墙视觉：
  - 卡片悬浮抬起（轻微 transform + shadow）
  - 卡片间距 16px
- [ ] **9.5** 主题变更立即同步到所有便签窗口（通过 event 广播）
- [ ] **9.6** 字体：系统默认字体栈（`-apple-system, "Segoe UI", "Microsoft YaHei", sans-serif`）
- [ ] **9.7** 图标集成：将阶段 13（图标生成）的产物放入 `src-tauri/icons/`，配置 `tauri.conf.json` 引用

### 验收点
- 浅色/深色主题切换无重渲染卡顿
- 跟随系统模式随系统切换
- 所有便签窗口主题一致
- 卡片墙视觉与 PRD 6.3 节示意一致

---

## 阶段 10：健壮性与备份

### 目标
落地 PRD 5.3 节健壮性要求。

### 任务清单

- [ ] **10.1** 落盘失败重试：`save_notes` 失败重试 3 次，间隔 100ms，仍失败则 `emit("storage:error")` 通知前端
- [ ] **10.2** 崩溃恢复：启动时检测 `data.json` 是否合法 JSON，损坏则尝试从 `backups/` 最近一份恢复
- [ ] **10.3** 路径兜底：D 盘检测失败的单元测试
- [ ] **10.4** 自动备份触发：每次 `save_notes` 成功后，若 `auto_backup = true`，调 `backup()`
- [ ] **10.5** 备份清理：`backup()` 完成后删除超过 `backup_retention` 的旧备份
- [ ] **10.6** 多窗口并发：所有 commands 通过 `State<Mutex<AppState>>` 串行化，无竞态
- [ ] **10.7** 前端错误提示：`storage:error` 事件 → 主窗口顶部 toast 提示「保存失败」
- [ ] **10.8** 全局错误边界：前端 `onError` 捕获未处理异常，日志输出

### 验收点
- 手动破坏 `data.json`，启动应用能从备份恢复
- `auto_backup = true` 时编辑便签后 `backups/` 出现新文件
- 备份超过 retention 数量时最旧的被删除
- 模拟落盘失败（临时改权限），前端有提示

---

## 阶段 11：测试

### 目标
覆盖核心路径，确保 MVP 质量。

### 任务清单

- [ ] **11.1** Rust 单元测试：
  - `storage` 模块：路径解析、加载/保存、备份
  - `commands` 模块：CRUD 业务逻辑
- [ ] **11.2** 前端单元测试（Vitest）：
  - `lib/url.ts` URL 识别
  - `stores/notes.ts` 状态变更
- [ ] **11.3** 集成测试（手动 checklist）：
  - 创建便签 → 编辑 → 改色 → 关闭 → 重新贴出 → 数据一致
  - 主窗口关闭 → 托盘仍在 → 便签仍在
  - 全局快捷键新建
  - 重启应用恢复 pinned 便签
  - 主题切换
  - 导出/导入往返
  - 删除确认
- [ ] **11.4** 性能基准测试（对照 PRD 5.1 节）：
  - 安装包体积
  - 空载内存
  - 典型使用内存（5 便签）
  - 启动时间
  - 1000 条便签列表加载时间
- [ ] **11.5** 边界测试：
  - 0 便签
  - 50 便签同时贴出
  - 超长正文（10KB）
  - 特殊字符（emoji、CRLF）
  - D 盘不存在的回退

### 验收点
- `cargo test` 全通过
- `pnpm test` 全通过
- 集成测试 checklist 全部 ✓
- 性能指标全部达到 PRD 5.1 节目标

---

## 阶段 12：打包与发布

### 目标
产出可分发的 Windows 安装包。

### 任务清单

- [ ] **12.1** 配置 `tauri.conf.json` `bundle`：
  - `active: true`
  - `targets: ["msi", "nsis"]`（任选其一或两者）
  - `icon`: 引用 `icons/` 下的多尺寸图标
  - `windows.webviewInstallMode`: `downloadBootstrapper`（处理 WebView2 未装情况）
- [ ] **12.2** 配置应用版本号：`version: "1.0.0"`
- [ ] **12.3** 配置 `productName`、`copyright`、`publisher` 等元信息
- [ ] **12.4** 执行构建：`tauri build`（或 `pnpm tauri build`）
- [ ] **12.5** 验证产物：
  - `src-tauri/target/release/bundle/msi/*.msi` 体积 ≤ 10MB
  - `src-tauri/target/release/bundle/nsis/*.exe` 体积 ≤ 10MB
- [ ] **12.6** 干净 Windows 环境安装测试（虚拟机或另一台机器）
- [ ] **12.7** 安装后验证：
  - 桌面/开始菜单出现 DeskNote
  - 启动正常
  - 数据目录正确创建
- [ ] **12.8** 卸载测试：卸载后用户数据是否保留（建议保留，方便用户重装）

### 验收点
- 安装包体积 ≤ 10MB
- 干净环境安装可正常运行
- 卸载行为符合预期

---

## 阶段 13：MVP 验收

### 目标
对照 PRD 7.2 节 DoD 逐项验收。

### 任务清单

- [ ] **13.1** 可在 Windows 10/11 上安装运行，安装包 ≤ 10 MB
- [ ] **13.2** 通过主窗口网格卡片墙创建、编辑、删除、贴出、收回便签
- [ ] **13.3** 便签窗口置顶、无标题栏、可拖动、可调整大小、不占任务栏
- [ ] **13.4** 关闭主窗口最小化到托盘，便签仍贴在桌面
- [ ] **13.5** 全局快捷键 `Ctrl+Alt+N` 快速新建便签
- [ ] **13.6** 6 色板颜色、位置记忆、纯文本 + URL 识别
- [ ] **13.7** 设置项全部可用（自启、快捷键、默认颜色、数据目录显示、自动备份、主题、导出/导入、关闭行为）
- [ ] **13.8** 数据持久化到 `D:\ProgramData\notes_data\`（D 盘不存在时回退 appDataDir）
- [ ] **13.9** 性能指标全部达标（PRD 5.1 节）
- [ ] **13.10** 系统托盘菜单四项可用

### 验收点
- 全部 10 项 ✓
- 打 git tag `v1.0.0`
- MVP 交付完成

---

## 附录 A：常用命令速查

```bash
# 开发
tauri dev                         # 启动开发模式（全局 CLI）
pnpm tauri dev                    # 等价写法（项目本地 CLI）
tauri build                       # 构建生产包
cargo test                        # Rust 测试
pnpm test                         # 前端测试

# 代码质量
cargo fmt                         # Rust 格式化
cargo clippy                      # Rust lint
pnpm lint                         # 前端 lint
pnpm format                       # 前端格式化

# 调试
tauri info                        # 查看 Tauri 环境信息
```

> **命令说明**：Tauri CLI 通过 `pnpm add -g @tauri-apps/cli@^2` 全局安装，可直接用 `tauri xxx`。
> 脚手架创建项目时也会自动把 `@tauri-apps/cli` 加为项目 devDependency，因此 `pnpm tauri xxx` 也可用，且版本锁定更可靠。两种写法等价，任选其一。

## 附录 B：依赖清单（前端 ≤ 10 个）

**dependencies（运行时）**

| 依赖 | 用途 | 必要性 |
|---|---|---|
| `@tauri-apps/api` | Tauri 前端 API | 必需 |
| `@tauri-apps/plugin-autostart` | 开机自启 | 必需 |
| `@tauri-apps/plugin-global-shortcut` | 全局快捷键 | 必需 |
| `@tauri-apps/plugin-dialog` | 文件对话框（导出/导入） | 必需 |
| `@tauri-apps/plugin-fs` | 文件读写（导出/导入） | 必需 |
| `svelte` | 前端框架 | 必需 |
| `uuid` | 生成 UUID v4 | 必需 |

**devDependencies（开发时，不计入运行时 bundle）**

| 依赖 | 用途 | 必要性 |
|---|---|---|
| `@tauri-apps/cli` | Tauri CLI（开发命令） | 必需 |
| `vite` | 构建工具 | 必需 |
| `vitest` | 测试框架 | 必需 |

> **说明**：PRD 5.2 节「前端 dependencies ≤ 10 个」约束的是运行时 dependencies（影响 bundle 体积），devDependencies 不计入。当前 dependencies = 7 个，预留 3 个额度给未来需求。

---

**文档结束**
