# DeskNote 便签完成功能设计方案（优化文档 2）

> 基于 v1.1 拟物化版本，新增「便签完成」状态与交互。
> 核心目标：从「只能创建/编辑/删除」升级为「完成生命周期闭环」——未完成 ↔ 已完成，桌面端 & 主窗口双入口操作。

---

## 一、现状分析与差距总览

### 1.1 当前便签状态机（缺失）

当前便签仅有二维状态：

| 维度 | 取值 | 说明 |
|------|------|------|
| `pinned` | `true / false` | 是否贴在桌面（窗口存在与否） |
| （无） | —— | **缺失「是否已完成」的语义维度** |

导致的问题：
- 用户做完一件事，只能「删除」或「放在那不处理」，没有「标记完成但保留记录」的中间态
- 主窗口越积越多做完的便签，视觉噪音大
- 无法复盘「本周完成了哪些事」

### 1.2 新增维度

便签状态机升级为三维：

```
Note {
  pinned:       bool       ← 现有：是否贴出
  completed_at: Option<i64> ← 新增：None=未完成 / Some(ts)=已完成(时间戳)
  ...
}
```

交叉后的 4 种合法状态：

| 状态 | `pinned` | `completed_at` | 说明 |
|------|----------|----------------|------|
| 未完成 + 收回 | `false` | `None` | 主窗口「未完成」Tab |
| 未完成 + 贴出 | `true` | `None` | 桌面上的便签，按钮空心灰✓ |
| 已完成 + 收回 | `false` | `Some(ts)` | 主窗口「已完成」Tab |
| 已完成 + 贴出 | `true` | `Some(ts)` | 已完成再次贴出，按钮实心绿✓（可撤销） |

### 1.3 交互缺口清单

| 缺口 | 当前 | 目标 |
|------|------|------|
| 便签窗口完成按钮 | 无 | 顶部把手右侧：`[⋮⋮ 标题...] [✓] [×]` |
| 主窗口卡片完成按钮 | 无 | 卡片悬浮出现 `[✓完成]` / `[↩️撤销完成]` |
| 主窗口视图切换 | 单一卡片墙 | 顶部 Tab：`📋 未完成` / `✅ 已完成(N)` |
| 完成后视觉反馈 | 无 | 按钮变绿 + 删除线 + 淡出动画 + Toast 提示 |
| 完成事件广播 | 无 | 新增 `note:completed` / `note:uncompleted` 跨窗口同步 |
| 排序/时间展示 | 按 `updated_at` | 已完成 Tab 按 `completed_at` 倒序，卡片显示「完成于 3 分钟前」 |

---

## 二、数据模型变更

### 2.1 Rust 侧（types.rs）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    // ... 现有 12 个字段不变 ...
    pub id: String,
    pub title: String,
    pub content: String,
    pub color: NoteColor,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
    pub rotation: f32,
    pub monitor: String,

    /// 完成时间戳（Unix ms），None = 未完成
    /// serde default：旧 data.json 缺失字段默认为 None（未完成）
    #[serde(default)]
    pub completed_at: Option<i64>,
}
```

**向后兼容说明**：
- `#[serde(default)]` 对 `Option<T>` 默认值就是 `None`，旧数据 100% 兼容
- 导入/导出 JSON 自动包含该字段

### 2.2 前端侧（types/note.ts）

```typescript
export interface Note {
  // ... 现有字段不变 ...
  id: string;
  title: string;
  content: string;
  color: NoteColor;
  width: number;
  height: number;
  x: number;
  y: number;
  created_at: number;
  updated_at: number;
  pinned: boolean;
  rotation?: number;
  monitor?: string;

  /** 完成时间戳（Unix ms），undefined = 未完成 */
  completed_at?: number;
}
```

---

## 三、Rust 后端实现方案

### 3.1 新增 Commands（commands.rs）

```rust
/// 标记便签为已完成
/// 写入 completed_at = now；若 pinned=true 则顺便置为 false（收回桌面）
/// 成功后 emit note:completed 事件
#[tauri::command]
pub async fn complete_note(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<Note, DeskNoteError> {
    let mut note = state.complete(&id)?;  // 内部写 completed_at + touch()
    if note.pinned {
        note.pinned = false;
        let _ = state.set_pinned(&id, false)?;
        // 关闭便签窗口（若存在）
        let label = format!("note-{}", id);
        if let Some(win) = app.get_webview_window(&label) {
            let _ = win.close();
        }
    }
    app.emit(event_name::NOTE_COMPLETED, NotePayload { id: note.id.clone() })?;
    Ok(note)
}

/// 撤销完成（变回未完成），completed_at = None
/// pinned 状态保持不变（贴出的便签不收回）
/// 成功后 emit note:uncompleted 事件
#[tauri::command]
pub async fn uncomplete_note(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<Note, DeskNoteError> {
    let note = state.uncomplete(&id)?;
    app.emit(event_name::NOTE_UNCOMPLETED, NotePayload { id: note.id.clone() })?;
    Ok(note)
}
```

### 3.2 AppState 新增方法（state.rs）

```rust
impl AppState {
    /// 标记完成，返回修改后的 Note
    pub fn complete(&self, id: &str) -> Result<Note, DeskNoteError> {
        let mut inner = self.inner.lock().unwrap();
        let note = inner.notes.get_mut(id)
            .ok_or(DeskNoteError::NoteNotFound(id.to_string()))?;
        if note.completed_at.is_none() {
            note.completed_at = Some(chrono::Utc::now().timestamp_millis());
            note.touch();
            inner.mark_dirty();
        }
        Ok(note.clone())
    }

    /// 撤销完成
    pub fn uncomplete(&self, id: &str) -> Result<Note, DeskNoteError> {
        let mut inner = self.inner.lock().unwrap();
        let note = inner.notes.get_mut(id)
            .ok_or(DeskNoteError::NoteNotFound(id.to_string()))?;
        if note.completed_at.is_some() {
            note.completed_at = None;
            note.touch();
            inner.mark_dirty();
        }
        Ok(note.clone())
    }
}
```

### 3.3 注册到 invoke_handler（lib.rs）

在 `generate_handler![]` 中追加：
```rust
commands::complete_note,
commands::uncomplete_note,
```

### 3.4 新增事件常量（events.rs）

```rust
pub mod event_name {
    // ... 现有常量不变 ...
    pub const NOTE_COMPLETED: &str = "note:completed";
    pub const NOTE_UNCOMPLETED: &str = "note:uncompleted";
}
```

---

## 四、前端实现方案

### 4.1 commands 封装（lib/commands.ts）

```typescript
/** 标记完成（桌面便签会自动收回） */
export function completeNote(id: string): Promise<Note> {
  return invoke('complete_note', { id });
}

/** 撤销完成（不收回便签窗口） */
export function uncompleteNote(id: string): Promise<Note> {
  return invoke('uncomplete_note', { id });
}
```

### 4.2 events 封装（lib/events.ts）

```typescript
type NoteCompletedCb = (payload: { id: string }) => void;

export function onNoteCompleted(cb: NoteCompletedCb): Promise<UnlistenFn> {
  return listen('note:completed', (event) => cb(event.payload as { id: string }));
}

export function onNoteUncompleted(cb: NoteCompletedCb): Promise<UnlistenFn> {
  return listen('note:uncompleted', (event) => cb(event.payload as { id: string }));
}
```

### 4.3 stores 订阅（stores/notes.ts）

在 `refresh()` 方法和事件订阅区，新增：
```typescript
// onNoteCreated / onNoteUpdated / onNoteDeleted 之后追加
onNoteCompleted(({ id }) => {
  notes.update(list => list.map(n =>
    n.id === id ? { ...n, completed_at: Date.now(), pinned: false } : n
  ));
});

onNoteUncompleted(({ id }) => {
  notes.update(list => list.map(n =>
    n.id === id ? { ...n, completed_at: undefined } : n
  ));
});
```

---

## 五、UI 实现方案（分 4 个阶段）

### 阶段 1：便签窗口完成按钮（NoteApp.svelte）

**影响文件**：`src/views/NoteApp.svelte`

#### 1.1 按钮位置：顶部把手右侧

布局结构（从左到右）：
```
[⋮⋮ 拖动把手] [标题输入框............] [✓完成按钮] [×关闭按钮]
```

#### 1.2 完成按钮样式

```svelte
<script>
  // 派生状态
  $: isCompleted = !!note?.completed_at;
  let isCompletingAnim = false; // 播放收回动画标记
</script>

<!-- 顶部把手区 -->
<div class="handle" data-tauri-drag-region>
  <span class="drag-icon">⋮⋮</span>

  <input
    class="title-input"
    bind:value={title}
    placeholder="标题（可选）"
    on:blur={saveTitle}
  />

  <!-- 完成按钮（新） -->
  <button
    class="complete-btn"
    class:completed={isCompleted}
    on:click={handleCompleteToggle}
    title={isCompleted ? '撤销完成' : '标记完成并收回'}
    aria-label={isCompleted ? '撤销完成' : '标记完成'}
  >
    {#if isCompleted}
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
    {:else}
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
    {/if}
  </button>

  <!-- 关闭按钮（现有） -->
  <button class="close-btn" on:click={handleClose} aria-label="收回便签">×</button>
</div>
```

#### 1.3 完成按钮 CSS

```css
.complete-btn {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 1.5px solid var(--note-title);
  opacity: 0;  /* 默认隐藏，hover 时淡入（与关闭按钮一致） */
  background: transparent;
  color: var(--note-title);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-right: 6px;
  transition: all 150ms ease;
  flex-shrink: 0;
}

/* 未完成态 hover */
.complete-btn:not(.completed):hover {
  background: #4CAF50;  /* 柔和绿 */
  border-color: #4CAF50;
  color: white;
  opacity: 1 !important;
}

/* 已完成态：绿色实心 */
.complete-btn.completed {
  background: #4CAF50;
  border-color: #4CAF50;
  color: white;
  opacity: 1 !important;
}

/* 父容器 hover 时显示完成按钮 / 关闭按钮 */
.handle:hover .complete-btn,
.note-app:focus-within .complete-btn,
.complete-btn.completed {  /* 已完成态常显 */
  opacity: 0.85;
}
.handle:hover .close-btn,
.note-app:focus-within .close-btn {
  opacity: 0.85;
}
```

#### 1.4 完成/撤销完成交互逻辑

```svelte
<script>
  let isCompletingAnim = false;

  async function handleCompleteToggle() {
    if (!note || isCompletingAnim) return;

    if (isCompleted) {
      // 已完成 → 撤销完成（不收回窗口）
      await uncompleteNote(note.id);
      // store 会通过 event 自动更新，note.completed_at 变为 undefined
    } else {
      // 未完成 → 已完成（播放动画 + 收回）
      isCompletingAnim = true;
      // 1. 立即打标视觉（按钮变绿 + 删除线 + 半透明）
      //    由 isCompleted 通过 event 回来时切换为 true 已在 store 层处理
      //    但为了动画流畅，这里手动加一个 completing class
      // 2. 调后端（后端会自动关闭窗口 + emit event）
      try {
        await completeNote(note.id);
      } catch (e) {
        isCompletingAnim = false;
        console.error('complete failed', e);
      }
    }
  }
</script>

<!-- note-app 根节点新增 class 绑定 -->
<main
  class="note-app"
  class:completing={isCompletingAnim || isCompleted}
  ...
>
```

#### 1.5 完成状态的便签视觉（完成态样式）

```css
/* 完成态：删除线 + 半透明 */
.note-app.completing .title-input,
.note-app.completing .content-textarea {
  text-decoration: line-through;
  text-decoration-thickness: 1.5px;
  text-decoration-color: color-mix(in srgb, var(--note-title) 50%, transparent);
}

.note-app.completing {
  opacity: 0.7;
  transition: opacity 200ms ease, transform 300ms ease;
}

/* 完成并收回的缩小淡出动画（仅针对未完成→已完成那次点击） */
.note-app.completing.closing-anim {
  transform: scale(0.85);
  opacity: 0;
}
```

#### 1.6 Toast 提示（NoteApp.svelte 内）

完成后需要 toast 反馈。但 Toast 组件建议在主窗口（MainApp.svelte）统一处理，因为主窗口一直存在。便签窗口完成后即将关闭，不适合显示 Toast。

方案：**主窗口订阅 `note:completed` / `note:uncompleted` 事件后显示统一 Toast。**

---

### 阶段 2：主窗口顶部 Tab 切换（MainApp.svelte）

**影响文件**：`src/views/MainApp.svelte`

#### 2.1 Tab 栏 UI

在顶部栏（搜索框所在行下方）插入 Tab 栏：

```
┌───────────────────────────────────────────────────────┐
│ DeskNote     🔍[搜索框]          [⚙设置]  [+新建]    │  ← 原顶部栏
├───────────────────────────────────────────────────────┤
│  📋 未完成    │    ✅ 已完成 (12)                      │  ← 新增 Tab 栏
├───────────────────────────────────────────────────────┤
│  卡片墙（按当前 Tab 过滤）                              │
└───────────────────────────────────────────────────────┘
```

```svelte
<script>
  type TabKey = 'active' | 'completed';
  let currentTab: TabKey = 'active';

  // 派生过滤后的数据
  $: filteredNotes = ($notes ?? [])
    .filter(n => {
      if (currentTab === 'active') return !n.completed_at;
      return !!n.completed_at;
    })
    .filter(n => {
      // 原有搜索逻辑
      if (!searchQuery.trim()) return true;
      const q = searchQuery.toLowerCase();
      return (n.title + n.content).toLowerCase().includes(q);
    })
    .sort((a, b) => {
      // 未完成 Tab：按 updated_at 倒序（现有逻辑）
      // 已完成 Tab：按 completed_at 倒序
      if (currentTab === 'completed') {
        return (b.completed_at || 0) - (a.completed_at || 0);
      }
      return b.updated_at - a.updated_at;
    });

  // 已完成数量（徽标）
  $: completedCount = ($notes ?? []).filter(n => !!n.completed_at).length;
</script>

<!-- Tab 栏 -->
<div class="tab-bar">
  <button
    class="tab-item"
    class:active={currentTab === 'active'}
    on:click={() => currentTab = 'active'}
  >
    <span class="tab-icon">📋</span>
    <span>未完成</span>
  </button>
  <button
    class="tab-item"
    class:active={currentTab === 'completed'}
    on:click={() => currentTab = 'completed'}
  >
    <span class="tab-icon">✅</span>
    <span>已完成</span>
    {#if completedCount > 0}
      <span class="tab-badge">({completedCount})</span>
    {/if}
  </button>
</div>
```

#### 2.2 Tab 样式

```css
.tab-bar {
  display: flex;
  gap: 4px;
  padding: 4px 24px 0 24px;
  border-bottom: 1px solid var(--border);
}

.tab-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px; /* 跟 bottom border 重合 */
  transition: all 150ms ease;
  border-radius: 6px 6px 0 0;
}

.tab-item:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.tab-item.active {
  color: var(--primary);
  border-bottom-color: var(--primary);
  font-weight: 600;
}

.tab-icon {
  font-size: 15px;
}

.tab-badge {
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 400;
}
```

#### 2.3 空状态文案

```svelte
<!-- 现有空状态，根据 currentTab 区分文案 -->
{#if filteredNotes.length === 0}
  <div class="empty-state">
    {#if currentTab === 'active'}
      <div class="empty-icon">📝</div>
      <div class="empty-title">暂无便签</div>
      <div class="empty-desc">点击右上角「+ 新建」创建第一张便签</div>
    {:else}
      <div class="empty-icon">✅</div>
      <div class="empty-title">还没有完成的便签</div>
      <div class="empty-desc">在便签上点击 ✓ 按钮标记完成</div>
    {/if}
  </div>
{/if}
```

---

### 阶段 3：卡片完成按钮 + 视觉区分（NoteCard.svelte）

**影响文件**：`src/components/NoteCard.svelte`

#### 3.1 卡片组件新增 props + 状态

```svelte
<script>
  export let note: Note;

  $: isCompleted = !!note.completed_at;
  let showActions = false;  // hover 显示操作按钮

  // 相对时间（完成于 / 更新于）
  $: timeLabel = (() => {
    if (isCompleted && note.completed_at) {
      return `完成于 ${formatRelative(note.completed_at)}`;
    }
    return formatRelative(note.updated_at);
  })();
</script>
```

#### 3.2 卡片完成态视觉

```svelte
<div
  class="card"
  class:completed={isCompleted}
  on:mouseenter={() => showActions = true}
  on:mouseleave={() => showActions = false}
>
  <!-- 顶部胶条（完成态变灰） -->
  <div
    class="color-bar"
    style="--card-color: var(--color-{note.color});
           --card-color-tape: var(--color-{note.color}-tape);"
  ></div>

  <div class="card-body">
    <div class="card-title" class:strikethrough={isCompleted}>
      {note.title || getPreview(note.content, 30) || '（无标题）'}
    </div>
    <div class="card-preview" class:strikethrough={isCompleted}>
      {getPreview(note.content, 60) || '（空内容）'}
    </div>
    <div class="card-meta">
      {timeLabel}
      {#if note.pinned}
        <span class="pin-badge">📌 贴出</span>
      {/if}
    </div>

    <!-- 悬浮操作按钮 -->
    {#if showActions}
      <div class="card-actions">
        {#if !isCompleted}
          <button
            class="action-btn complete"
            on:click|stopPropagation={handleComplete}
            title="标记完成"
          >
            ✓ 完成
          </button>
          <button
            class="action-btn pin"
            on:click|stopPropagation={handlePinToggle}
          >
            {note.pinned ? '收回' : '贴出'}
          </button>
        {:else}
          <button
            class="action-btn uncomplete"
            on:click|stopPropagation={handleUncomplete}
            title="撤销完成"
          >
            ↩️ 撤销
          </button>
          <button
            class="action-btn pin"
            on:click|stopPropagation={handlePinToggle}
          >
            贴出
          </button>
        {/if}
        <button
          class="action-btn delete"
          on:click|stopPropagation={handleDelete}
          title="删除"
        >
          🗑️
        </button>
      </div>
    {/if}
  </div>
</div>
```

#### 3.3 完成态样式

```css
.card.completed {
  opacity: 0.75;
}

.card.completed .color-bar {
  filter: grayscale(0.6);
}

.strikethrough {
  text-decoration: line-through;
  text-decoration-thickness: 1px;
  text-decoration-color: color-mix(in srgb, var(--text-secondary) 60%, transparent);
}

/* 操作按钮：完成/撤销 颜色区分 */
.action-btn.complete {
  color: #4CAF50;
}
.action-btn.complete:hover {
  background: color-mix(in srgb, #4CAF50 12%, transparent);
}

.action-btn.uncomplete {
  color: #F5A623;  /* 复用品牌色 */
}
.action-btn.uncomplete:hover {
  background: color-mix(in srgb, #F5A623 12%, transparent);
}
```

#### 3.4 卡片事件处理

```svelte
<script>
  import { completeNote, uncompleteNote, pinNote, unpinNote, deleteNote } from '$lib/commands';
  import { emit } from '@tauri-apps/api/event';

  async function handleComplete() {
    try {
      await completeNote(note.id);
      // Toast 通过主窗口订阅 event 显示，此处无需处理
    } catch (e) {
      console.error('complete failed', e);
    }
  }

  async function handleUncomplete() {
    try {
      await uncompleteNote(note.id);
    } catch (e) {
      console.error('uncomplete failed', e);
    }
  }

  // 现有 handlePinToggle / handleDelete 不变
</script>
```

---

### 阶段 4：Toast 提示（MainApp.svelte）

**影响文件**：`src/views/MainApp.svelte`

#### 4.1 Toast 状态 + UI

```svelte
<script>
  interface Toast {
    id: number;
    type: 'success' | 'info' | 'error';
    message: string;
  }
  let toasts: Toast[] = [];
  let toastSeq = 0;

  function showToast(message: string, type: Toast['type'] = 'success', duration = 3000) {
    const id = ++toastSeq;
    toasts.push({ id, type, message });
    setTimeout(() => {
      toasts = toasts.filter(t => t.id !== id);
    }, duration);
  }

  // 订阅完成事件显示 Toast
  onMount(async () => {
    // ... 现有其他订阅 ...

    await onNoteCompleted(({ id }) => {
      showToast('✅ 便签已完成，可在「已完成」Tab 查看');
    });

    await onNoteUncompleted(({ id }) => {
      showToast('↩️ 已撤销完成');
    });
  });
</script>

<!-- Toast 容器 -->
<div class="toast-container">
  {#each toasts as toast (toast.id)}
    <div class="toast" class:success={toast.type === 'success'}>
      {toast.message}
    </div>
  {/each}
</div>
```

#### 4.2 Toast CSS

```css
.toast-container {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10000;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast {
  padding: 10px 20px;
  border-radius: 8px;
  background: #32302C;
  color: #E8E5DD;
  font-size: 14px;
  box-shadow: var(--shadow-strong);
  animation: toast-in 200ms ease, toast-out 200ms ease 2.8s forwards;
  pointer-events: auto;
}

.toast.success {
  background: linear-gradient(180deg, #5CB85C 0%, #4CAF50 100%);
  color: white;
}

@keyframes toast-in {
  from { opacity: 0; transform: translateY(-8px); }
  to { opacity: 1; transform: translateY(0); }
}
@keyframes toast-out {
  to { opacity: 0; transform: translateY(-8px); }
}
```

---

## 六、相对时间工具增强（lib/time.ts）

当前 `formatRelative` 基于 `updated_at` 即可，无需改动。已完成 Tab 显示「完成于 xxx」是在卡片里传不同的时间戳给同一个函数。

---

## 七、跨窗口同步时序

### 场景 A：桌面上便签 → 点完成按钮

```
NoteApp 点击 ✓
    │
    ├─ 1. 本地 class:completing（删除线+半透明）
    │
    ├─ 2. invoke complete_note(id) → Rust
    │       │
    │       ├─ 写 Note.completed_at = now
    │       ├─ Note.pinned = false
    │       ├─ 标记 dirty → 防抖落盘
    │       ├─ 关闭 note-{id} 窗口
    │       └─ emit note:completed → 所有窗口
    │
    ├─ 3. MainApp 收到 note:completed
    │       ├─ notes store 更新（该便签 completed_at 有值，pinned=false）
    │       └─ show Toast「便签已完成...」
    │
    └─ 4. 其他便签窗口（如其他便签贴出）收到 note:completed → store 更新
```

### 场景 B：主窗口卡片 → 悬浮点完成

```
NoteCard 悬浮 → 点「✓ 完成」
    │
    ├─ 1. invoke complete_note(id) → Rust
    │       ├─ 若该便签 pinned=true（正贴在桌面）
    │       │    └─ Rust 关闭 note-{id} 窗口
    │       └─ emit note:completed
    │
    ├─ 2. 桌面便签窗口（若存在）收到关闭指令 → 播放缩小淡出动画后关闭
    │
    └─ 3. MainApp 更新 store + show Toast
```

### 场景 C：已完成便签 → 再次贴出 → 撤销完成

```
已完成 Tab → 点卡片「贴出」→ pin_note(id) → 创建窗口
    │
    └─ 窗口加载该便签（completed_at 有值）→ 完成按钮显示绿✓实心
         │
         └─ 用户点击绿✓
              │
              ├─ invoke uncomplete_note(id) → Rust
              │     ├─ completed_at = None
              │     ├─ pinned 保持 true（不收回）
              │     └─ emit note:uncompleted
              │
              ├─ NoteApp：按钮变灰空心，删除线消失，opacity 恢复
              │
              └─ MainApp：store 更新（便签从已完成 Tab 回到未完成 Tab）
                   └─ show Toast「↩️ 已撤销完成」
```

---

## 八、实施清单

### 阶段 0：数据模型 + Commands / Events
- [ ] `types.rs`：Note 新增 `completed_at: Option<i64>` + `#[serde(default)]`
- [ ] `types/note.ts`：Note 接口新增 `completed_at?: number`
- [ ] `state.rs`：AppState 新增 `complete()` / `uncomplete()` 方法 + 单元测试
- [ ] `commands.rs`：新增 `complete_note` / `uncomplete_note` command
- [ ] `lib.rs`：注册两个新 command 到 `invoke_handler`
- [ ] `events.rs`：新增 `NOTE_COMPLETED` / `NOTE_UNCOMPLETED` 常量

### 阶段 1：便签窗口（NoteApp.svelte）
- [ ] 顶部把手 HTML：在 `[×]` 左侧插入 `[✓]` 完成按钮
- [ ] CSS：`.complete-btn` 未完成 / 已完成 / hover 三态样式
- [ ] 隐藏/显示规则：与关闭按钮一致（hover/focus 显示，已完成态常显）
- [ ] 完成态视觉：`.completing` 删除线 + opacity 0.7
- [ ] 缩小淡出动画：调 complete_note 后播 300ms 动画
- [ ] 交互逻辑：`handleCompleteToggle()` 完成/撤销双向

### 阶段 2：主窗口 Tab 切换（MainApp.svelte）
- [ ] 顶部 Tab 栏：`📋 未完成` / `✅ 已完成(N)` 两个 Tab
- [ ] 数据过滤：`currentTab` 控制 `filteredNotes` 的 completed_at 过滤
- [ ] 排序规则：未完成按 updated_at 倒序，已完成按 completed_at 倒序
- [ ] 空状态区分文案
- [ ] Toast 组件：容器 + 动画 + showToast()
- [ ] onMount 订阅 `note:completed` / `note:uncompleted` → Toast

### 阶段 3：卡片（NoteCard.svelte）
- [ ] 完成态样式：`.completed` 卡片 opacity 0.75 + 胶条 grayscale + 文字删除线
- [ ] 时间显示：已完成显示「完成于 xxx」，否则显示「更新于 xxx」
- [ ] 悬浮操作按钮：未完成→「✓完成 / 贴出 / 🗑️」，已完成→「↩️撤销 / 贴出 / 🗑️」
- [ ] `handleComplete()` / `handleUncomplete()` 调用对应 commands

### 阶段 4：前端基础库
- [ ] `commands.ts`：export `completeNote()` / `uncompleteNote()`
- [ ] `events.ts`：export `onNoteCompleted()` / `onNoteUncompleted()`
- [ ] `stores/notes.ts`：订阅两个新 event → 更新 store

### 阶段 5：验证
- [ ] `pnpm check`：0 errors / 0 warnings
- [ ] `cargo test`：Rust 侧新增 tests 全部通过
- [ ] `pnpm test`：前端单测（若有 stores 相关）
- [ ] 手动验证：6 大场景走通（见下节）

---

## 九、手动验证 Checklist（6 大场景）

| # | 场景 | 预期结果 |
|---|------|---------|
| 1 | 桌面便签点✓ | 按钮变绿→文字删除线→缩小淡出→窗口关闭→主窗口 Toast「已完成」→ 已完成 Tab +1 |
| 2 | 主窗口未完成卡片，悬浮点「✓完成」 | Toast 提示→卡片从当前 Tab 消失→若正贴出，桌面窗口同步关闭→已完成 Tab +1 |
| 3 | 已完成 Tab → 点卡片「贴出」 | 窗口出现→完成按钮绿✓实心→文字有删除线+半透明 |
| 4 | 已完成贴出的便签，点绿✓（撤销） | 按钮变灰空心→删除线消失→窗口不消失→回到未完成 Tab→Toast「撤销完成」 |
| 5 | 已完成 Tab → 悬浮点「↩️撤销」 | 卡片从已完成 Tab 消失→出现在未完成 Tab→Toast「撤销完成」 |
| 6 | 旧数据（无 completed_at 字段）启动后正常 | 所有便签默认在「未完成」Tab，功能正常 |

---

## 十、性能约束

| 指标 | 约束 | 说明 |
|------|------|------|
| 新增 DOM 节点（便签窗口） | +1 个 button | 完成按钮，可接受 |
| 新增 DOM 节点（主窗口） | +2 Tab 按钮 + Toast 容器 | 可忽略 |
| 动画时长 | ≤ 300ms | 完成收回动画，超过会感觉慢 |
| 新增 CSS 类 | ≤ 10 个 | 全部为 scoped / 组件级 |
| 新增 Rust fields | 1 个 Option<i64> | 单便签 +8 bytes，1000 张 +8KB，可忽略 |
| 跨窗口事件 | +2 个 | completed / uncompleted |

---

## 十一、验收标准

1. **功能完整性**：9.1-9.6 手动验证场景全部通过
2. **数据一致性**：完成后刷新 App，已完成状态保留；导入导出 JSON 包含 completed_at
3. **向后兼容**：旧 data.json（无 completed_at 字段）启动后所有便签在「未完成」Tab
4. **视觉一致性**：完成态删除线 / 半透明 / 绿按钮在所有 6 色便签上清晰可读
5. **代码质量**：`pnpm check` 0 errors / 0 warnings + `cargo test` 全部通过
6. **性能不退化**：拖动便签流畅度、主窗口卡片墙加载速度与优化前持平
