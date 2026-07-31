<script lang="ts">
  // 主窗口组件（阶段 4 占位：渲染 notes store 数量验证通信链路）
  // 阶段 6 实现完整网格卡片墙
  import { notes, notesLoading } from '../stores/notes';
  import { config } from '../stores/config';
  import { createNote } from '../lib/commands';
  import { getPreviewText } from '../lib/url';

  async function handleCreate() {
    await createNote({ title: '', content: '新便签 ' + new Date().toLocaleTimeString() });
    // 事件订阅会自动 refresh，无需手动调用
  }
</script>

<main class="main-app">
  <h1>DeskNote</h1>
  <p>轻量级桌面便签应用</p>

  <div class="status">
    {#if $notesLoading}
      <p class="placeholder">加载中…</p>
    {:else}
      <p>便签数量：{$notes.length}</p>
      <p>主题：{$config.theme}</p>
      <p>快捷键：{$config.hotkey}</p>
    {/if}
  </div>

  <button class="primary" on:click={handleCreate}>创建便签（测试）</button>

  {#if $notes.length > 0}
    <ul class="note-list">
      {#each $notes as note (note.id)}
        <li>
          <span class="color-dot" style="background: var(--color-{note.color})"></span>
          <span class="preview">{getPreviewText(note)}</span>
          <span class="time">{new Date(note.updated_at).toLocaleString()}</span>
          {#if note.pinned}<span class="pinned-badge">已贴出</span>{/if}
        </li>
      {/each}
    </ul>
  {/if}
</main>

<style>
  .main-app {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-6);
    height: 100%;
    overflow: auto;
  }

  h1 {
    font-size: 2rem;
    font-weight: 600;
    color: var(--accent);
    margin-bottom: var(--space-1);
  }

  p {
    color: var(--fg-secondary);
  }

  .status {
    margin: var(--space-5) 0;
    padding: var(--space-4);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    min-width: 280px;
    text-align: center;
  }

  .placeholder {
    color: var(--fg-tertiary);
  }

  .note-list {
    list-style: none;
    width: 100%;
    max-width: 600px;
    margin-top: var(--space-5);
  }

  li {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--border);
  }

  .color-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .preview {
    flex: 1;
    color: var(--fg);
  }

  .time {
    font-size: 0.75rem;
    color: var(--fg-tertiary);
  }

  .pinned-badge {
    font-size: 0.7rem;
    padding: 2px 6px;
    background: var(--accent);
    color: var(--accent-fg);
    border-radius: var(--radius-sm);
  }
</style>
