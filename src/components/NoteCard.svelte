<script lang="ts">
  // 便签卡片组件（SOP 6.4-6.6）
  // 顶部色条 + 标题 + 正文预览 + 相对时间 + 悬浮操作按钮
  import { createEventDispatcher } from 'svelte';
  import type { Note } from '../types/note';
  import { getPreviewText } from '../lib/url';
  import { relativeTime } from '../lib/time';

  export let note: Note;

  const dispatch = createEventDispatcher();

  $: previewTitle = getPreviewText(note);
  $: contentPreview = note.content.trim().replace(/\s+/g, ' ');

  function onDoubleClick() {
    dispatch('edit', note);
  }

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    dispatch('contextmenu', { x: e.clientX, y: e.clientY, note });
  }

  function onTogglePin(e: MouseEvent) {
    e.stopPropagation();
    dispatch('togglePin', note);
  }

  function onDelete(e: MouseEvent) {
    e.stopPropagation();
    dispatch('delete', note);
  }
</script>

<div
  class="card"
  class:pinned={note.pinned}
  style="--card-color: var(--color-{note.color}); --card-color-deep: var(--color-{note.color}-deep);"
  on:dblclick={onDoubleClick}
  on:contextmenu={onContextMenu}
  role="button"
  tabindex="0"
  title="双击编辑 · 右键菜单"
>
  <div class="color-bar"></div>

  <div class="body">
    <h3 class="title">{previewTitle}</h3>
    {#if contentPreview}
      <p class="content">{contentPreview}</p>
    {/if}
  </div>

  <div class="footer">
    <span class="time">{relativeTime(note.updated_at)}</span>
    {#if note.pinned}
      <span class="pinned-tag">已贴出</span>
    {/if}
  </div>

  <div class="actions">
    <button
      class="action-btn"
      on:click={onTogglePin}
      title={note.pinned ? '收回' : '贴出'}
      aria-label={note.pinned ? '收回' : '贴出'}
    >
      {note.pinned ? '▾' : '📍'}
    </button>
    <button
      class="action-btn danger"
      on:click={onDelete}
      title="删除"
      aria-label="删除"
    >
      ✕
    </button>
  </div>
</div>

<style>
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    aspect-ratio: 4 / 3;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
    cursor: pointer;
    transition: box-shadow var(--transition), transform var(--transition), border-color var(--transition);
    outline: none;
  }

  .card:hover,
  .card:focus-visible {
    box-shadow: var(--shadow-strong);
    transform: translateY(-2px);
    border-color: var(--border-strong);
  }

  .card.pinned {
    border-color: var(--card-color-deep);
  }

  /* 顶部色条 */
  .color-bar {
    height: 6px;
    background: var(--card-color);
    border-bottom: 2px solid var(--card-color-deep);
    flex-shrink: 0;
  }

  .body {
    flex: 1;
    padding: var(--space-2) var(--space-3);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--fg);
    line-height: 1.3;
    /* 单行截断 */
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .content {
    font-size: 0.8rem;
    color: var(--fg-secondary);
    line-height: 1.4;
    /* 2 行截断 */
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-1) var(--space-3) var(--space-2);
    flex-shrink: 0;
  }

  .time {
    font-size: 0.7rem;
    color: var(--fg-tertiary);
  }

  .pinned-tag {
    font-size: 0.65rem;
    padding: 1px 6px;
    background: var(--card-color);
    color: var(--fg);
    border-radius: var(--radius-sm);
  }

  /* 悬浮操作按钮 */
  .actions {
    position: absolute;
    top: var(--space-2);
    right: var(--space-2);
    display: flex;
    gap: var(--space-1);
    opacity: 0;
    transition: opacity var(--transition);
  }

  .card:hover .actions,
  .card:focus-within .actions {
    opacity: 1;
  }

  .action-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.85);
    color: var(--fg-secondary);
    font-size: 0.75rem;
    line-height: 1;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(4px);
    transition: all var(--transition);
  }

  .action-btn:hover {
    background: var(--bg);
    color: var(--fg);
  }

  .action-btn.danger:hover {
    background: var(--danger);
    color: #fff;
  }
</style>
