<script lang="ts">
  // 便签窗口主组件（阶段 5）
  // PRD 6.2 / SOP 5：拖动、关闭、编辑、改色、尺寸/位置记忆、多窗口同步
  // 阶段 9：订阅 config:updated，主题变更实时同步到便签窗口
  import { onMount, onDestroy } from 'svelte';
  import {
    getCurrentWindow,
    currentMonitor as apiCurrentMonitor,
  } from '@tauri-apps/api/window';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import type { Note, NoteColor } from '../types/note';
  import { NOTE_COLORS } from '../types/note';
  import { getNotes, updateNote, unpinNote } from '../lib/commands';
  import { onNoteUpdated, onNoteDeleted, onNoteUnpinned } from '../lib/events';
  import { splitByUrl } from '../lib/url';
  import {
    config,
    refreshConfig,
    startConfigSubscription,
    stopConfigSubscription,
  } from '../stores/config';

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

  // v1.1 拟物化 1.2：拖动抬升状态
  // Tauri 2.x 无 onMoveStart，用首次 onMoved 触发 + dragIdleTimer 静止判定结束
  let dragging = false;
  let dragIdleTimer: ReturnType<typeof setTimeout> | null = null;

  // v1.1 拟物化 1.4/1.6：进出场动画状态
  // entering：窗口 show 后淡入 + 缩放放大；leaving：收回前反向缩放淡出
  let entering = false;
  let leaving = false;

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
    // v1.1 拟物化 1.5：收回前反向缩放淡出动画（200ms）
    leaving = true;
    await new Promise((r) => setTimeout(r, 200));
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
    // v1.1 拟物化 1.2：首次 onMoved 进入 dragging 状态（抬升阴影 + scale），
    //   300ms 静止判定拖动结束，回落阴影
    // v1.1 优化阶段 2.4：记录显示器标识（拖动结束后才查询，避免每像素 IPC 卡顿）
    //   同时更新 cachedScaleFactor，避免跨显示器 DPI 不同导致坐标换算错误
    try {
      let lastMonitorName = '';
      unlistens.push(
        await win.onMoved(async (e) => {
          if (!noteReady) return;

          const { x, y } = e.payload;
          pendingPos = {
            x: Math.round(x / cachedScaleFactor),
            y: Math.round(y / cachedScaleFactor),
          };

          // 拖动抬升：首次 onMoved 置 true，重置静止计时器
          if (!dragging) dragging = true;
          if (dragIdleTimer) clearTimeout(dragIdleTimer);
          dragIdleTimer = setTimeout(async () => {
            dragging = false;
            // 拖动结束后才查询显示器标识（避免每像素移动都发 IPC）
            try {
              const mon = await apiCurrentMonitor();
              if (mon) {
                const name = mon.name ?? '';
                // 更新 cachedScaleFactor（跨显示器 DPI 可能不同）
                if (mon.scaleFactor !== cachedScaleFactor) {
                  cachedScaleFactor = mon.scaleFactor;
                }
                // 用最新的 scaleFactor 重新校准 pendingPos（如果有未提交的位置）
                if (pendingPos) {
                  pendingPos = {
                    x: Math.round(e.payload.x / cachedScaleFactor),
                    y: Math.round(e.payload.y / cachedScaleFactor),
                  };
                  void flushPosition();
                }
                // 保存显示器标识（变化时才落库）
                if (name !== lastMonitorName) {
                  lastMonitorName = name;
                  if (note) {
                    note.monitor = name;
                    updateNote(note.id, { monitor: name }).catch((err) =>
                      console.error('[DeskNote] 保存显示器失败', err),
                    );
                  }
                }
              }
            } catch {
              /* ignore */
            }
          }, 300);
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

    // 初始化：若便签 monitor 为空（新便签或旧数据迁移），立即记录当前显示器，
    // 并确保位置坐标是正确的逻辑像素（避免首次贴出后立即收回再贴出丢失位置）
    if (note && !note.monitor) {
      try {
        const mon = await apiCurrentMonitor();
        if (mon) {
          const name = mon.name ?? '';
          if (name) {
            note.monitor = name;
            updateNote(note.id, { monitor: name }).catch((err) =>
              console.error('[DeskNote] 初始化显示器失败', err),
            );
          }
          // 用当前显示器的 scaleFactor 刷新 cachedScaleFactor
          if (mon.scaleFactor !== cachedScaleFactor) {
            cachedScaleFactor = mon.scaleFactor;
          }
        }
      } catch {
        /* ignore */
      }
    }

    // v1.1 拟物化 1.4/1.6：show 前先置 entering（初始 scale 0.9 + opacity 0），
    //   show 后下一帧移除 entering，触发 250ms 淡入 + 缩放过渡（贴出飞入感）
    entering = true;
    try {
      await win.show();
      await win.setFocus();
      // 下一帧移除 entering 触发过渡（双重 rAF 确保浏览器先渲染初始状态）
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          entering = false;
        });
      });
    } catch {
      /* ignore */
    }

    // 阶段 9：启动 config 订阅，主题变更实时同步到本便签窗口
    // v1.1 优化阶段 2.2：订阅 always_on_top 变更动态设置窗口置顶
    try {
      await refreshConfig();
      await startConfigSubscription();
      // 初始值 + 响应式订阅
      let lastAot = $config.always_on_top ?? true;
      win.setAlwaysOnTop(lastAot).catch(() => {/* ignore */});
      const unsubConfig = config.subscribe((c) => {
        const aot = c.always_on_top ?? true;
        if (aot !== lastAot) {
          lastAot = aot;
          win.setAlwaysOnTop(aot).catch(() => {/* ignore */});
        }
      });
      unlistens.push(unsubConfig);
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
    if (dragIdleTimer) clearTimeout(dragIdleTimer);
    stopConfigSubscription();
  });

  // 便签多层级色彩随 color 变化（v2.0 拟物化：底色/胶条/标题/正文/卷角）
  $: noteBg = note ? `var(--color-${note.color})` : 'var(--color-yellow)';
  $: noteTape = note ? `var(--color-${note.color}-tape)` : 'var(--color-yellow-tape)';
  $: noteTitleColor = note ? `var(--color-${note.color}-title)` : 'var(--color-yellow-title)';
  $: noteContentColor = note ? `var(--color-${note.color}-content)` : 'var(--color-yellow-content)';
  $: noteFold = note ? `var(--color-${note.color}-fold)` : 'var(--color-yellow-fold)';
  // 桌面便签随机微旋转（v1.1 拟物化，±2° 模拟随手贴）
  $: rotation = note?.rotation ?? 0;
</script>

<main
  class="note-app"
  class:dragging
  class:entering
  class:leaving
>
  <div
    class="note-app-inner"
    style="--note-bg: {noteBg}; --note-tape: {noteTape}; --note-title: {noteTitleColor}; --note-content: {noteContentColor}; --note-fold: {noteFold}; --rotation: {rotation}deg;"
  >
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
  </div>
</main>

<style>
  /* 便签窗口：body 透明，由便签容器承载背景 + 圆角 + 阴影（SOP 5.8） */
  :global(html),
  :global(body) {
    background: transparent;
  }

  /* 外层容器：透明，仅负责窗口级裁切（100vw/100vh） */
  .note-app {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: transparent;
    user-select: none;
  }

  /* v2.4 拟物化：三层光影结构
   * 内层：inset 高光（顶部+左侧），体现纸张受光面厚度
   * 中层：贴身浅阴影，模拟纸张自身厚度投影
   * 外层：大范围柔阴影，体现悬浮在桌面上的扩散投影 */
  .note-app-inner {
    position: absolute;
    top: -6px;
    left: -6px;
    right: -6px;
    bottom: -6px;
    display: flex;
    flex-direction: column;
    background-color: var(--note-bg);
    background-image:
      radial-gradient(circle at 20% 30%, rgba(180, 170, 150, 0.02) 0%, transparent 50%),
      radial-gradient(circle at 70% 60%, rgba(180, 170, 150, 0.02) 0%, transparent 50%),
      radial-gradient(circle at 40% 80%, rgba(180, 170, 150, 0.015) 0%, transparent 50%);
    border-radius: var(--radius-lg);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.35),
      inset 1px 0 0 rgba(255, 255, 255, 0.15),
      0 1px 2px rgba(120, 100, 40, 0.08),
      0 6px 16px rgba(120, 100, 40, 0.10),
      0 14px 32px rgba(120, 100, 40, 0.12);
    transform: rotate(var(--rotation, 0deg));
    transform-origin: center center;
    overflow: hidden;
    transition: background-color 300ms ease, box-shadow 200ms ease, transform 250ms ease,
      opacity 250ms ease;
  }

  /* v2.4 拖动抬升 —— 三层阴影同步放大 + 轻微 scale */
  .note-app.dragging .note-app-inner {
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.35),
      inset 1px 0 0 rgba(255, 255, 255, 0.15),
      0 2px 4px rgba(120, 100, 40, 0.12),
      0 10px 24px rgba(120, 100, 40, 0.14),
      0 20px 48px rgba(120, 100, 40, 0.16);
    transform: rotate(var(--rotation, 0deg)) scale(1.02);
    transition: none;
  }

  /* 新建/贴出淡入 */
  .note-app.entering .note-app-inner {
    opacity: 0;
    transform: rotate(var(--rotation, 0deg)) scale(0.9);
  }

  /* 收回反向缩放淡出 */
  .note-app.leaving .note-app-inner {
    opacity: 0;
    transform: rotate(var(--rotation, 0deg)) scale(0.92);
  }

  /* v2.4 拟物化：右下角卷角 — 22px + 背面深色 + 内侧投影 + 折痕高光
   * ① 背面色比底色深 15%，区分纸张正反面
   * ② 内侧投影：左上方淡阴影，模拟翘起后投在便签正面的影子
   * v2.5 真实卷角：径向渐变曲面 + 折痕高光 + 多层翘起投影 */
  .note-app-inner::after {
    content: '';
    position: absolute;
    right: 0;
    bottom: 0;
    /* v2.5.2 向左上翘起更多：宽 40 × 高 32（不等腰），斜边更平缓，
     *   纸角向左上方延伸更远，视觉上翘起感更强 */
    width: 40px;
    height: 32px;
    clip-path: polygon(100% 0, 100% 100%, 0 100%);
    /* 径向渐变：折痕根部深色 → 翘起边缘略亮，模拟曲面光照
     * v2.5.1 加深对比：根部 35%+黑（更深），边缘 75%+白（更亮），曲面感更明显 */
    background:
      radial-gradient(
        ellipse at 100% 100%,
        color-mix(in srgb, var(--note-fold) 35%, #000) 0%,
        color-mix(in srgb, var(--note-fold) 60%, #000) 35%,
        var(--note-fold) 60%,
        color-mix(in srgb, var(--note-fold) 75%, #fff) 100%
      );
    /* 折痕高光（inset 亮边）+ 翘起纸角投在正面的阴影（外阴影）
     * v2.5.1 同步加深内侧暗角 0.14 → 0.22
     * v2.5.2 配合更大卷角，外阴影偏移与模糊同步放大 */
    box-shadow:
      inset 1px 1px 0 rgba(255, 255, 255, 0.50),
      inset -1px -1px 3px rgba(0, 0, 0, 0.22),
      -3px -3px 4px rgba(0, 0, 0, 0.10),
      -6px -6px 12px rgba(60, 50, 20, 0.15);
    border-bottom-right-radius: var(--radius-lg);
    pointer-events: none;
    z-index: 3;
  }

  /* v2.5 卷角桌面阴影：翘起纸角投在桌面上的扩散投影（远景柔阴影）
   * v2.5.2 配合 40×32 卷角，阴影区扩大并向左下方延伸更多 */
  .note-app-inner::before {
    content: '';
    position: absolute;
    right: -6px;
    bottom: -6px;
    width: 54px;
    height: 46px;
    background: radial-gradient(
      circle at 75% 75%,
      rgba(60, 50, 20, 0.22) 0%,
      rgba(60, 50, 20, 0.11) 32%,
      transparent 68%
    );
    pointer-events: none;
    z-index: 0;
  }

  /* 让 handle/content/toolbar 在伪元素之上 */
  .note-app-inner > :global(*),
  .note-app-inner > * {
    position: relative;
    z-index: 2;
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

  /* v2.4 拟物化：胶条厚度 — 微渐变 + 顶部受光亮边 + 底部压痕暗线
   * 渐变：顶部 tape 原色（受光）→ 底部 tape×80%+bg 混合（压痕），符合斜上方光照 */
  .handle {
    height: 40px;
    display: flex;
    align-items: center;
    padding: 0 var(--space-3);
    gap: var(--space-1);
    flex-shrink: 0;
    cursor: grab;
    background: linear-gradient(
      180deg,
      var(--note-tape) 0%,
      color-mix(in srgb, var(--note-tape) 80%, var(--note-bg)) 100%
    );
    /* 顶部 1px 亮边（受光面）+ 底部 1px 暗线（压痕），模拟胶条压在纸上的厚度 */
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.45),
      inset 0 -1px 0 rgba(0, 0, 0, 0.10);
    transition: background var(--transition), filter var(--transition);
  }

  .handle:hover {
    background: linear-gradient(
        180deg,
        var(--note-tape) 0%,
        color-mix(in srgb, var(--note-tape) 80%, var(--note-bg)) 100%
      );
    filter: brightness(0.97);
  }

  .handle:active {
    cursor: grabbing;
    background: linear-gradient(
        180deg,
        var(--note-tape) 0%,
        color-mix(in srgb, var(--note-tape) 80%, var(--note-bg)) 100%
      );
    filter: brightness(0.94);
  }

  /* 拖动图标（⋮⋮）
   * v1.1 拟物化 1.1：默认隐藏，hover/focus-within 时淡入 */
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
    opacity: 0;
    transition: opacity 200ms ease, color var(--transition);
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
    font-weight: 700;
    color: var(--note-title);
    padding: 0 var(--space-2);
    margin: 0 var(--space-1);
    border-radius: var(--radius-sm);
    user-select: text;
  }

  .title-input::placeholder {
    color: var(--note-title);
    opacity: 0.4;
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
    /* v1.1 拟物化 1.1：默认隐藏，hover/focus-within 时淡入 */
    opacity: 0;
    transition: opacity 200ms ease, background var(--transition), color var(--transition);
  }

  .close-btn:hover {
    background: var(--danger);
    color: #fff;
  }

  /* 正文区（SOP 5.4）
   * v2.1 拟物化：便签专属正文色 + 加大内边距 + line-height 1.7 模拟手写松弛感 */
  .content {
    flex: 1;
    border: none;
    background: transparent;
    resize: none;
    padding: var(--space-3) var(--space-4);
    font-family: var(--font-sans);
    font-size: 0.875rem;
    line-height: 1.7;
    letter-spacing: 0.01em;
    color: var(--note-content);
    user-select: text;
    outline: none;
  }

  .content::placeholder {
    color: var(--note-content);
    opacity: 0.4;
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
    /* v1.1 拟物化 1.1：默认隐藏，hover/focus-within 时淡入 */
    opacity: 0;
    transition: opacity 200ms ease;
  }

  /* v1.1 拟物化 1.1：便签 hover 或内部 focus 时，按钮/色板/拖动图标淡入显示
   *   标题输入框始终可见（核心交互） */
  .note-app:hover .close-btn,
  .note-app:hover .palette,
  .note-app:hover .drag-grip,
  .note-app:focus-within .close-btn,
  .note-app:focus-within .palette,
  .note-app:focus-within .drag-grip {
    opacity: 1;
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
