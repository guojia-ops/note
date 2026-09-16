<script lang="ts">
  // 主窗口（SOP 6.1-6.10）
  // 顶部栏 + 网格卡片墙 + 卡片交互 + 右键菜单 + 编辑模态 + 空状态
  // 阶段 8：监听托盘/全局快捷键的 tray:new-note 事件
  import { onMount, onDestroy } from 'svelte';
  import { notes, notesLoading } from '../stores/notes';
  import {
    createNote,
    pinNote,
    unpinNote,
    deleteNote,
    updateNote,
    getNotes,
    completeNote,
    uncompleteNote,
  } from '../lib/commands';
  import { onTrayNewNote, onStorageError, onNoteCompleted, onNoteUncompleted } from '../lib/events';
  import NoteCard from '../components/NoteCard.svelte';
  import NoteEditModal from '../components/NoteEditModal.svelte';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
  import type { Note } from '../types/note';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  const trayUnlistens: UnlistenFn[] = [];

  // SOP 10.7：落盘失败 toast 提示
  let storageError = '';
  let storageErrorTimer: ReturnType<typeof setTimeout> | null = null;
  function showStorageError(msg: string) {
    storageError = `保存失败：${msg}`;
    if (storageErrorTimer) clearTimeout(storageErrorTimer);
    // 5 秒后自动消失
    storageErrorTimer = setTimeout(() => (storageError = ''), 5000);
  }

  // 搜索
  let keyword = '';
  let searchTerm = '';
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  function onSearchInput() {
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      searchTerm = keyword.trim().toLowerCase();
    }, 200);
  }

  // Tab 切换：未完成 / 已完成
  type TabKey = 'active' | 'completed';
  let currentTab: TabKey = 'active';
  $: completedCount = $notes.filter((n) => !!n.completed_at).length;

  // 过滤后的便签列表
  // - 按 Tab 过滤完成状态
  // - 按搜索词过滤
  // - 排序：未完成 Tab → pinned 优先 + updated_at 倒序；已完成 Tab → completed_at 倒序
  $: filtered = $notes
    .filter((n) => (currentTab === 'active' ? !n.completed_at : !!n.completed_at))
    .filter((n) => {
      if (!searchTerm) return true;
      return (
        n.title.toLowerCase().includes(searchTerm) ||
        n.content.toLowerCase().includes(searchTerm)
      );
    })
    .sort((a, b) => {
      if (currentTab === 'completed') {
        return (b.completed_at || 0) - (a.completed_at || 0);
      }
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
      return b.updated_at - a.updated_at;
    });

  // 通用 Toast 提示
  interface Toast {
    id: number;
    type: 'success' | 'info' | 'error';
    message: string;
  }
  let toasts: Toast[] = [];
  let toastSeq = 0;
  let toastTimers: Map<number, ReturnType<typeof setTimeout>> = new Map();

  function showToast(message: string, type: Toast['type'] = 'success', duration = 3000) {
    const id = ++toastSeq;
    toasts = [...toasts, { id, type, message }];
    const t = setTimeout(() => {
      toasts = toasts.filter((x) => x.id !== id);
      toastTimers.delete(id);
    }, duration);
    toastTimers.set(id, t);
  }

  // 新建便签：创建数据 + 立即贴出
  async function handleCreate() {
    const note = await createNote({
      title: '',
      content: '',
    });
    await pinNote(note.id);
  }

  // 打开设置窗口（SOP 7.1）
  function handleSettings() {
    const label = 'settings';
    const win = new WebviewWindow(label, {
      url: 'settings.html',
      title: 'DeskNote 设置',
      width: 560,
      height: 680,
      minWidth: 480,
      minHeight: 480,
      resizable: true,
      decorations: true,
      transparent: false,
    });
    win.once('tauri://created', () => {
      win.setFocus().catch(() => {});
    });
    win.once('tauri://error', (e) => {
      console.error('[DeskNote] 设置窗口创建失败', e);
    });
  }

  // 编辑模态
  let editingNote: Note | null = null;
  function openEdit(note: Note) {
    editingNote = note;
  }
  function closeEdit() {
    editingNote = null;
  }
  async function saveEdit(e: CustomEvent<{ id: string; title: string; content: string }>) {
    const { id, title, content } = e.detail;
    await updateNote(id, { title, content });
    closeEdit();
  }

  // 贴出/收回
  async function togglePin(note: Note) {
    if (note.pinned) {
      await unpinNote(note.id);
    } else {
      await pinNote(note.id);
    }
  }

  // 删除（带确认）
  async function handleDelete(note: Note) {
    const ok = confirm(`确定删除该便签？\n\n${note.title || note.content.slice(0, 30) || '（空便签）'}`);
    if (!ok) return;
    // 若已贴出，先收回关闭窗口
    if (note.pinned) {
      try {
        await unpinNote(note.id);
      } catch {
        /* ignore */
      }
    }
    await deleteNote(note.id);
  }

  // 卡片：标记完成（从未完成 Tab）
  async function handleComplete(note: Note) {
    try {
      await completeNote(note.id);
    } catch (e) {
      console.error('[DeskNote] 标记完成失败', e);
      showToast('标记完成失败', 'error');
    }
  }

  // 卡片：撤销完成（从已完成 Tab）
  async function handleUncomplete(note: Note) {
    try {
      await uncompleteNote(note.id);
    } catch (e) {
      console.error('[DeskNote] 撤销完成失败', e);
      showToast('撤销完成失败', 'error');
    }
  }

  // 右键菜单
  let menuX = 0;
  let menuY = 0;
  let menuNote: Note | null = null;

  function openMenu(e: { x: number; y: number; note: Note }) {
    menuX = e.x;
    menuY = e.y;
    menuNote = e.note;
  }

  function closeMenu() {
    menuNote = null;
  }

  function menuTogglePin() {
    if (menuNote) void togglePin(menuNote);
    closeMenu();
  }

  function menuEdit() {
    if (menuNote) openEdit(menuNote);
    closeMenu();
  }

  function menuComplete() {
    if (menuNote) {
      if (menuNote.completed_at) {
        void handleUncomplete(menuNote);
      } else {
        void handleComplete(menuNote);
      }
    }
    closeMenu();
  }

  function menuDelete() {
    if (menuNote) void handleDelete(menuNote);
    closeMenu();
  }

  // SOP 8.8：监听托盘菜单/全局快捷键触发的「新建便签」事件
  // SOP 10.7：监听 storage:error 事件，显示 toast
  // 应用重启后恢复已贴出的便签窗口
  onMount(async () => {
    try {
      trayUnlistens.push(
        await onTrayNewNote(() => {
          void handleCreate();
        }),
      );
    } catch (e) {
      console.error('[DeskNote] tray:new-note 监听注册失败', e);
    }
    try {
      trayUnlistens.push(await onStorageError(showStorageError));
    } catch (e) {
      console.error('[DeskNote] storage:error 监听注册失败', e);
    }
    // 完成/撤销完成 Toast 反馈
    try {
      trayUnlistens.push(
        await onNoteCompleted(() => {
          showToast('✅ 便签已完成，可在「已完成」Tab 查看');
        }),
      );
    } catch (e) {
      console.error('[DeskNote] note:completed 监听注册失败', e);
    }
    try {
      trayUnlistens.push(
        await onNoteUncompleted(() => {
          showToast('↩️ 已撤销完成', 'info');
        }),
      );
    } catch (e) {
      console.error('[DeskNote] note:uncompleted 监听注册失败', e);
    }

    // 恢复已贴出的便签窗口：应用重启后，pinned=true 的便签需要重新创建窗口
    try {
      const list = await getNotes();
      const pinnedNotes = list.filter((n) => n.pinned);
      if (pinnedNotes.length > 0) {
        await Promise.all(
          pinnedNotes.map((n) =>
            pinNote(n.id).catch((e) =>
              console.error(`[DeskNote] 恢复便签 ${n.id} 失败`, e),
            ),
          ),
        );
      }
    } catch (e) {
      console.error('[DeskNote] 恢复便签失败', e);
    }
  });

  onDestroy(() => {
    trayUnlistens.forEach((u) => {
      try {
        u();
      } catch {
        /* ignore */
      }
    });
    if (storageErrorTimer) clearTimeout(storageErrorTimer);
    for (const t of toastTimers.values()) clearTimeout(t);
    toastTimers.clear();
  });

  // SOP 10.8：全局错误边界，捕获未处理异常并输出日志
  function handleWindowError(e: any) {
    console.error('[DeskNote] 未捕获异常:', e?.message, e?.error);
  }
  function handleUnhandledRejection(e: any) {
    console.error('[DeskNote] 未处理的 Promise 拒绝:', e?.reason);
  }
</script>

<svelte:window
  on:click={closeMenu}
  on:error={handleWindowError}
  on:unhandledrejection={handleUnhandledRejection}
/>

<main class="main-app">
  <!-- SOP 10.7：落盘失败 toast -->
  {#if storageError}
    <div class="storage-toast" role="alert">{storageError}</div>
  {/if}
  <!-- 通用 Toast 容器 -->
  <div class="toast-container">
    {#each toasts as toast (toast.id)}
      <div class="toast" class:success={toast.type === 'success'} class:error={toast.type === 'error'}>
        {toast.message}
      </div>
    {/each}
  </div>

  <!-- 顶部栏（SOP 6.2） -->
  <header class="topbar">
    <h1 class="brand">DeskNote</h1>
    <input
      class="search"
      type="text"
      placeholder={currentTab === 'completed' ? '搜索已完成便签…' : '搜索便签…'}
      bind:value={keyword}
      on:input={onSearchInput}
    />
    <button class="primary" on:click={handleCreate} title="新建便签">+ 新建</button>
    <button on:click={handleSettings} title="设置" aria-label="设置">⚙</button>
  </header>

  <!-- Tab 切换栏 -->
  <nav class="tab-bar" role="tablist" aria-label="便签状态分类">
    <button
      role="tab"
      aria-selected={currentTab === 'active'}
      class="tab-item"
      class:active={currentTab === 'active'}
      on:click={() => currentTab = 'active'}
    >
      <span class="tab-icon">📋</span>
      <span>未完成</span>
    </button>
    <button
      role="tab"
      aria-selected={currentTab === 'completed'}
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
  </nav>

  <!-- 内容区 -->
  <section class="content">
    {#if $notesLoading}
      <div class="state">加载中…</div>
    {:else if filtered.length === 0}
      <!-- 空状态（SOP 6.9） -->
      <div class="state empty">
        {#if searchTerm}
          <p>没有匹配的便签</p>
        {:else if currentTab === 'active'}
          <p class="empty-icon" aria-hidden="true">📝</p>
          <p class="empty-title">暂无未完成的便签</p>
          <p class="empty-hint">点击「+ 新建」创建第一张便签</p>
        {:else}
          <p class="empty-icon" aria-hidden="true">✅</p>
          <p class="empty-title">还没有完成的便签</p>
          <p class="empty-hint">在便签上点击 ✓ 按钮标记完成</p>
        {/if}
      </div>
    {:else}
      <!-- 网格卡片墙（SOP 6.3） -->
      <div class="grid">
        {#each filtered as note (note.id)}
          <NoteCard
            {note}
            on:edit={(e) => openEdit(e.detail)}
            on:togglePin={(e) => togglePin(e.detail)}
            on:delete={(e) => handleDelete(e.detail)}
            on:complete={(e) => handleComplete(e.detail)}
            on:uncomplete={(e) => handleUncomplete(e.detail)}
            on:contextmenu={(e) => openMenu(e.detail)}
          />
        {/each}
      </div>
    {/if}
  </section>
</main>

<!-- 编辑模态（SOP 6.8） -->
{#if editingNote}
  <NoteEditModal note={editingNote} on:save={saveEdit} on:cancel={closeEdit} />
{/if}

<!-- 右键菜单（SOP 6.5） -->
{#if menuNote}
  <div
    class="context-menu"
    style="left: {menuX}px; top: {menuY}px;"
    on:click|stopPropagation
    on:contextmenu|preventDefault
    on:keydown={(e) => e.key === 'Escape' && closeMenu()}
    role="menu"
    tabindex="-1"
  >
    <button class="menu-item" on:click={menuTogglePin} role="menuitem">
      {menuNote.pinned ? '▾ 收回' : '📍 贴出'}
    </button>
    <button class="menu-item" on:click={menuEdit} role="menuitem">✎ 编辑</button>
    <button class="menu-item" on:click={menuComplete} role="menuitem">
      {menuNote.completed_at ? '↩️ 撤销完成' : '✓ 标记完成'}
    </button>
    <div class="menu-divider"></div>
    <button class="menu-item danger" on:click={menuDelete} role="menuitem">✕ 删除</button>
  </div>
{/if}

<style>
  .main-app {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* 顶部栏 */
  .topbar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .brand {
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--accent);
    margin-right: var(--space-2);
    flex-shrink: 0;
  }

  .search {
    flex: 1;
    min-width: 0;
    height: 32px;
  }

  /* 内容区 */
  .content {
    flex: 1;
    overflow: auto;
    padding: var(--space-4);
  }

  /* 网格卡片墙（SOP 6.3） */
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--space-4); /* SOP 9.4 卡片间距 16px */
  }

  /* 状态展示 */
  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--fg-tertiary);
    text-align: center;
    gap: var(--space-1);
  }

  .empty-title {
    font-size: 1rem;
    color: var(--fg-secondary);
  }

  .empty-hint {
    font-size: 0.85rem;
  }

  .empty-icon {
    font-size: 2rem;
    margin-bottom: var(--space-2);
  }

  /* Tab 切换栏 */
  .tab-bar {
    display: flex;
    gap: 4px;
    padding: 0 var(--space-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .tab-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 10px 16px;
    border: none;
    background: transparent;
    color: var(--fg-secondary);
    font-size: 0.875rem;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: all 150ms ease;
    border-radius: 6px 6px 0 0;
  }

  .tab-item:hover {
    background: var(--bg-secondary);
    color: var(--fg-primary);
  }

  .tab-item.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
    font-weight: 600;
  }

  .tab-icon {
    font-size: 0.95rem;
  }

  .tab-badge {
    color: var(--fg-muted);
    font-size: 0.75rem;
    font-weight: 400;
  }

  /* 右键菜单 */
  .context-menu {
    position: fixed;
    z-index: 200;
    min-width: 140px;
    padding: var(--space-1);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-strong);
  }

  .menu-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: var(--space-2) var(--space-3);
    border: none;
    background: transparent;
    color: var(--fg);
    font-size: 0.85rem;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: background var(--transition);
  }

  .menu-item:hover {
    background: var(--bg-tertiary);
  }

  .menu-item.danger {
    color: var(--danger);
  }

  .menu-item.danger:hover {
    background: var(--danger);
    color: #fff;
  }

  .menu-divider {
    height: 1px;
    background: var(--border);
    margin: var(--space-1) 0;
  }

  /* SOP 10.7：落盘失败 toast */
  .storage-toast {
    position: fixed;
    top: var(--space-4);
    left: 50%;
    transform: translateX(-50%);
    background: var(--danger);
    color: #fff;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-strong);
    font-size: 0.875rem;
    z-index: 1000;
    animation: toast-in 200ms ease;
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }
  }

  /* 通用 Toast 容器（在 storage toast 下方堆叠） */
  .toast-container {
    position: fixed;
    top: 56px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 999;
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
    pointer-events: none;
  }

  .toast {
    padding: 10px 20px;
    border-radius: var(--radius-md);
    background: #32302C;
    color: #E8E5DD;
    font-size: 0.875rem;
    box-shadow: var(--shadow-strong);
    animation: toast-pop 200ms ease, toast-pop-out 200ms ease 2.8s forwards;
    pointer-events: auto;
  }

  .toast.success {
    background: linear-gradient(180deg, #5CB85C 0%, #4CAF50 100%);
    color: #fff;
  }

  .toast.error {
    background: linear-gradient(180deg, #e25a5a 0%, var(--danger) 100%);
    color: #fff;
  }

  @keyframes toast-pop {
    from {
      opacity: 0;
      transform: translateY(-6px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
  @keyframes toast-pop-out {
    to {
      opacity: 0;
      transform: translateY(-6px) scale(0.97);
    }
  }
</style>
