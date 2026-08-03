<script lang="ts">
  // 便签窗口主组件（阶段 5）
  // PRD 6.2 / SOP 5：拖动、关闭、编辑、改色、尺寸/位置记忆、多窗口同步
  // 阶段 9：订阅 config:updated，主题变更实时同步到便签窗口
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import type { Note, NoteColor } from '../types/note';
  import { NOTE_COLORS } from '../types/note';
  import { getNotes, updateNote, unpinNote } from '../lib/commands';
  import { onNoteUpdated, onNoteDeleted, onNoteUnpinned } from '../lib/events';
  import { splitByUrl } from '../lib/url';
  import { refreshConfig, startConfigSubscription, stopConfigSubscription } from '../stores/config';

  let note: Note | null = null;
  let loading = true;
  let errorMsg: string | null = null;
  let noteId: string | null = null;

  // 本地编辑状态
  let titleInput = '';
  let contentInput = '';
  let urlCount = 0;

  // 防抖定时器
  let contentTimer: ReturnType<typeof setTimeout> | null = null;
  let titleTimer: ReturnType<typeof setTimeout> | null = null;
  let moveTimer: ReturnType<typeof setTimeout> | null = null;
  let resizeTimer: ReturnType<typeof setTimeout> | null = null;

  // 待落库的尺寸/位置缓冲（关闭窗口前需立即 flush，避免丢失）
  let pendingSize: { w: number; h: number } | null = null;
  let pendingPos: { x: number; y: number } | null = null;

  // 缩放因子缓存（onResized/onMoved 同步转换物理像素→逻辑像素用）
  let cachedScaleFactor = 1;
  // 关闭流程守卫，避免 handleClose / onNoteUnpinned / onNoteDeleted 多路径重复关闭
  let closing = false;
  // note 数据是否已加载完成
  // 窗口创建时 Tauri 会立即触发 onResized/onMoved（初始化事件），此时 note 未加载，
  // 若处理会用窗口初始化尺寸/位置覆盖 note 的正确数据，因此必须忽略
  let noteReady = false;

  const unlistens: UnlistenFn[] = [];

  function countUrls(text: string): number {
    return splitByUrl(text).filter((s) => s.type === 'url').length;
  }

  async function loadNote() {
    try {
      const list = await getNotes();
      const found = list.find((n) => n.id === noteId) ?? null;
      if (!found) {
        errorMsg = '便签不存在或已删除';
        return;
      }
      note = found;
      titleInput = found.title;
      contentInput = found.content;
      urlCount = countUrls(contentInput);
    } catch (e) {
      errorMsg = String(e);
    } finally {
      loading = false;
    }
  }

  /** 立即提交未保存的正文 */
  async function flushContent() {
    if (contentTimer) {
      clearTimeout(contentTimer);
      contentTimer = null;
    }
    if (note && contentInput !== note.content) {
      try {
        const updated = await updateNote(note.id, { content: contentInput });
        if (note) note.content = updated.content;
      } catch (e) {
        console.error('[DeskNote] 保存正文失败', e);
      }
    }
  }

  /** 立即提交未保存的标题 */
  async function flushTitle() {
    if (titleTimer) {
      clearTimeout(titleTimer);
      titleTimer = null;
    }
    if (note && titleInput !== note.title) {
      try {
        const updated = await updateNote(note.id, { title: titleInput });
        if (note) note.title = updated.title;
      } catch (e) {
        console.error('[DeskNote] 保存标题失败', e);
      }
    }
  }

  /** 立即提交未保存的尺寸（SOP 5.6） */
  async function flushSize() {
    if (resizeTimer) {
      clearTimeout(resizeTimer);
      resizeTimer = null;
    }
    if (!note || !pendingSize) return;
    const { w, h } = pendingSize;
    pendingSize = null;
    if (w !== note.width || h !== note.height) {
      try {
        const updated = await updateNote(note.id, { width: w, height: h });
        if (note) {
          note.width = updated.width;
          note.height = updated.height;
        }
      } catch (e) {
        console.error('[DeskNote] 保存尺寸失败', e);
      }
    }
  }

  /** 立即提交未保存的位置（SOP 5.7） */
  async function flushPosition() {
    if (moveTimer) {
      clearTimeout(moveTimer);
      moveTimer = null;
    }
    if (!note || !pendingPos) return;
    const { x, y } = pendingPos;
    pendingPos = null;
    if (x !== note.x || y !== note.y) {
      try {
        const updated = await updateNote(note.id, { x, y });
        if (note) {
          note.x = updated.x;
          note.y = updated.y;
        }
      } catch (e) {
        console.error('[DeskNote] 保存位置失败', e);
      }
    }
  }

  /** 关闭窗口前统一 flush 所有未提交数据，避免调整/拖动后立即关闭导致丢失
   * 用 closing 标志守卫，防止 handleClose / onNoteUnpinned / onNoteDeleted 多路径重复触发 */
  async function closeWindow() {
    if (closing) return;
    closing = true;
    await Promise.all([flushContent(), flushTitle(), flushSize(), flushPosition()]);
    try {
      await getCurrentWindow().close();
    } catch {
      /* ignore */
    }
  }

  function onContentInput() {
    urlCount = countUrls(contentInput);
    if (contentTimer) clearTimeout(contentTimer);
    contentTimer = setTimeout(flushContent, 500);
  }

  function onTitleInput() {
    if (titleTimer) clearTimeout(titleTimer);
    titleTimer = setTimeout(flushTitle, 500);
  }

  /** 失焦立即保存标题（SOP 5.9） */
  function onTitleBlur() {
    if (titleTimer) {
      clearTimeout(titleTimer);
      titleTimer = null;
    }
    void flushTitle();
  }

  async function selectColor(c: NoteColor) {
    if (!note || note.color === c) return;
    try {
      const updated = await updateNote(note.id, { color: c });
      if (note) note.color = updated.color;
    } catch (e) {
      console.error('[DeskNote] 改色失败', e);
    }
  }

  async function handleClose() {
    if (closing) return;
    closing = true;
    // 关闭前 flush 所有未提交数据（正文/标题/尺寸/位置），避免丢失
    await Promise.all([flushContent(), flushTitle(), flushSize(), flushPosition()]);
    if (note) {
      try {
        // unpinNote：Rust set_pinned(false) + close 窗口 + emit note:unpinned
        // onNoteUnpinned 回调会再触发 closeWindow，closing 标志会跳过，避免重复关闭
        await unpinNote(note.id);
      } catch (e) {
        console.error('[DeskNote] 关闭失败', e);
        try {
          await getCurrentWindow().close();
        } catch {
          /* ignore */
        }
      }
    } else {
      try {
        await getCurrentWindow().close();
      } catch {
        /* ignore */
      }
    }
  }

  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    noteId = params.get('id');
    if (!noteId) {
      errorMsg = '缺少便签 id';
      loading = false;
      return;
    }

    await loadNote();

    const win = getCurrentWindow();

    // 缓存缩放因子，供 onResized/onMoved 同步转换物理像素→逻辑像素
    try {
      cachedScaleFactor = await win.scaleFactor();
    } catch {
      cachedScaleFactor = 1;
    }

    // SOP 5.6：监听调整大小，立即更新缓冲，防抖落库 width/height
    // 关键：pendingSize 必须同步设置，否则调整后 200ms 内关闭窗口会丢失
    // noteReady 守卫：忽略窗口创建时的初始化 onResized，避免覆盖 note 正确数据
    try {
      unlistens.push(
        await win.onResized((e) => {
          if (!noteReady) return;
          const { width, height } = e.payload;
          pendingSize = {
            w: Math.round(width / cachedScaleFactor),
            h: Math.round(height / cachedScaleFactor),
          };
          if (resizeTimer) clearTimeout(resizeTimer);
          resizeTimer = setTimeout(flushSize, 200);
        }),
      );
    } catch (e) {
      console.error('[DeskNote] onResized 注册失败', e);
    }

    // SOP 5.7：监听拖动结束，立即更新缓冲，防抖落库 x/y
    try {
      unlistens.push(
        await win.onMoved((e) => {
          if (!noteReady) return;
          const { x, y } = e.payload;
          pendingPos = {
            x: Math.round(x / cachedScaleFactor),
            y: Math.round(y / cachedScaleFactor),
          };
          if (moveTimer) clearTimeout(moveTimer);
          moveTimer = setTimeout(flushPosition, 200);
        }),
      );
    } catch (e) {
      console.error('[DeskNote] onMoved 注册失败', e);
    }

    // SOP 5.10：多窗口编辑一致性，订阅 note:updated
    try {
      unlistens.push(
        await onNoteUpdated(async (id) => {
          if (id !== noteId || !note) return;
          // 正在编辑的字段不覆盖，避免打断输入
          const active = document.activeElement;
          const editingContent = active?.tagName === 'TEXTAREA';
          const editingTitle = active?.classList.contains('title-input');
          try {
            const list = await getNotes();
            const fresh = list.find((n) => n.id === noteId);
            if (!fresh) return;
            note = fresh;
            if (!editingTitle) titleInput = fresh.title;
            if (!editingContent) {
              contentInput = fresh.content;
              urlCount = countUrls(contentInput);
            }
          } catch (e) {
            console.error('[DeskNote] 同步失败', e);
          }
        }),
      );
    } catch (e) {
      console.error('[DeskNote] onNoteUpdated 注册失败', e);
    }

    // 便签被删除 → 关闭窗口（先 flush，避免拖动/调整数据丢失）
    try {
      unlistens.push(
        await onNoteDeleted((id) => {
          if (id === noteId) void closeWindow();
        }),
      );
    } catch (e) {
      console.error('[DeskNote] onNoteDeleted 注册失败', e);
    }

    // 便签被收回（主窗口操作）→ 关闭窗口（先 flush，避免拖动/调整数据丢失）
    try {
      unlistens.push(
        await onNoteUnpinned((id) => {
          if (id === noteId) void closeWindow();
        }),
      );
    } catch (e) {
      console.error('[DeskNote] onNoteUnpinned 注册失败', e);
    }

    // 所有监听器注册完成，放行 onResized/onMoved，并显示窗口（避免空白闪烁）
    noteReady = true;
    try {
      await win.show();
      await win.setFocus();
    } catch {
      /* ignore */
    }

    // 阶段 9：启动 config 订阅，主题变更实时同步到本便签窗口
    try {
      await refreshConfig();
      await startConfigSubscription();
    } catch (e) {
      console.error('[DeskNote] 便签窗口 config 订阅失败', e);
    }
  });

  onDestroy(() => {
    unlistens.forEach((u) => {
      try {
        u();
      } catch {
        /* ignore */
      }
    });
    if (contentTimer) clearTimeout(contentTimer);
    if (titleTimer) clearTimeout(titleTimer);
    if (moveTimer) clearTimeout(moveTimer);
    if (resizeTimer) clearTimeout(resizeTimer);
    stopConfigSubscription();
  });

  // 便签背景色随 color 变化
  $: noteBg = note ? `var(--color-${note.color})` : 'var(--color-yellow)';
  $: noteBorder = note ? `var(--color-${note.color}-deep)` : 'var(--color-yellow-deep)';
</script>

<main class="note-app" style="--note-bg: {noteBg}; --note-border: {noteBorder};">
  {#if loading}
    <div class="state">加载中…</div>
  {:else if errorMsg}
    <div class="state error">{errorMsg}</div>
  {:else if note}
    <header class="handle" data-tauri-drag-region>
      <span class="drag-grip" data-tauri-drag-region title="拖动">⋮⋮</span>
      <input
        class="title-input"
        type="text"
        placeholder="标题（可空）"
        bind:value={titleInput}
        on:input={onTitleInput}
        on:blur={onTitleBlur}
      />
      <button class="close-btn" on:click={handleClose} title="收回便签" aria-label="收回便签">
        ×
      </button>
    </header>

    <textarea
      class="content"
      placeholder="写点什么…"
      bind:value={contentInput}
      on:input={onContentInput}
    ></textarea>

    <footer class="toolbar">
      <div class="palette">
        {#each NOTE_COLORS as c (c)}
          <button
            class="color-swatch"
            class:active={note.color === c}
            style="background: var(--color-{c}); border-color: var(--color-{c}-deep);"
            on:click={() => selectColor(c)}
            title={c}
            aria-label={c}
          ></button>
        {/each}
      </div>
      <div class="meta">
        {#if urlCount > 0}
          <span class="url-count">{urlCount} 个链接</span>
        {/if}
      </div>
    </footer>
  {/if}
</main>

<style>
  /* 便签窗口：body 透明，由便签容器承载背景 + 圆角 + 阴影（SOP 5.8） */
  :global(html),
  :global(body) {
    background: transparent;
  }

  .note-app {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background: var(--note-bg);
    border-radius: var(--radius-lg);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    overflow: hidden;
    user-select: none;
  }

  .state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-secondary);
    font-size: 0.875rem;
  }

  .state.error {
    color: var(--danger);
  }

  /* 顶部把手 40px（SOP 5.3，加宽便于拖动） */
  .handle {
    height: 40px;
    display: flex;
    align-items: center;
    padding: 0 var(--space-2);
    gap: var(--space-1);
    flex-shrink: 0;
    cursor: grab;
    transition: background var(--transition);
  }

  .handle:hover {
    background: rgba(0, 0, 0, 0.05);
  }

  .handle:active {
    cursor: grabbing;
    background: rgba(0, 0, 0, 0.08);
  }

  /* 拖动图标（⋮⋮） */
  .drag-grip {
    flex-shrink: 0;
    width: 16px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.7rem;
    letter-spacing: -2px;
    color: var(--fg-tertiary);
    cursor: grab;
    user-select: none;
  }

  .drag-grip:hover {
    color: var(--fg-secondary);
  }

  .handle:active .drag-grip {
    cursor: grabbing;
    color: var(--fg);
  }

  .title-input {
    flex: 1;
    min-width: 0;
    height: 28px;
    border: none;
    background: transparent;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--fg);
    padding: 0 var(--space-2);
    margin: 0 var(--space-1);
    border-radius: var(--radius-sm);
    user-select: text;
  }

  .title-input::placeholder {
    color: var(--fg-tertiary);
    font-weight: 400;
  }

  .title-input:focus {
    outline: none;
    background: rgba(255, 255, 255, 0.45);
  }

  .close-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--fg-secondary);
    font-size: 1.1rem;
    line-height: 1;
    padding: 0;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition);
  }

  .close-btn:hover {
    background: var(--danger);
    color: #fff;
  }

  /* 正文区（SOP 5.4） */
  .content {
    flex: 1;
    border: none;
    background: transparent;
    resize: none;
    padding: var(--space-2) var(--space-3);
    font-family: var(--font-sans);
    font-size: 0.875rem;
    line-height: 1.5;
    color: var(--fg);
    user-select: text;
    outline: none;
  }

  .content::placeholder {
    color: var(--fg-tertiary);
  }

  /* 底部工具条 36px（SOP 5.5） */
  .toolbar {
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-2);
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .palette {
    display: flex;
    gap: var(--space-1);
  }

  .color-swatch {
    width: 20px;
    height: 20px;
    padding: 0;
    border: 2px solid transparent;
    border-radius: 50%;
    cursor: pointer;
    transition: transform var(--transition);
  }

  .color-swatch:hover {
    transform: scale(1.15);
  }

  .color-swatch.active {
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.25);
  }

  .meta {
    display: flex;
    align-items: center;
  }

  .url-count {
    font-size: 0.7rem;
    color: var(--fg-tertiary);
  }
</style>
