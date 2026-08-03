<script lang="ts">
  // 编辑模态（SOP 6.8）：标题输入 + 正文 textarea + 保存/取消
  import { createEventDispatcher, onMount, tick } from 'svelte';
  import type { Note } from '../types/note';

  export let note: Note;

  const dispatch = createEventDispatcher();

  let titleInput = note.title;
  let contentInput = note.content;
  let contentEl: HTMLTextAreaElement;

  onMount(() => {
    // 自动聚焦正文
    tick().then(() => contentEl?.focus());
  });

  function save() {
    dispatch('save', { id: note.id, title: titleInput, content: contentInput });
  }

  function cancel() {
    dispatch('cancel');
  }

  function onBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) cancel();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') cancel();
    // Ctrl/Cmd + Enter 保存
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') save();
  }
</script>

<svelte:window on:keydown={onKey} />

<div class="backdrop" role="button" tabindex="-1" on:click={onBackdrop} on:keydown={(e) => e.key === 'Escape' && cancel()}>
  <div class="modal" role="dialog" aria-modal="true" aria-label="编辑便签">
    <header class="modal-header">
      <h2>编辑便签</h2>
      <button class="close-btn" on:click={cancel} aria-label="关闭">×</button>
    </header>

    <div class="modal-body">
      <label class="field">
        <span class="label">标题</span>
        <input
          class="input"
          type="text"
          bind:value={titleInput}
          placeholder="标题（可空）"
          maxlength="100"
        />
      </label>

      <label class="field">
        <span class="label">正文</span>
        <textarea
          class="input content-area"
          bind:value={contentInput}
          bind:this={contentEl}
          placeholder="写点什么…"
        ></textarea>
      </label>
    </div>

    <footer class="modal-footer">
      <span class="hint">Ctrl+Enter 保存 · Esc 取消</span>
      <div class="btns">
        <button on:click={cancel}>取消</button>
        <button class="primary" on:click={save}>保存</button>
      </div>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    backdrop-filter: blur(2px);
  }

  .modal {
    width: 90%;
    max-width: 480px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-strong);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border);
  }

  h2 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--fg);
  }

  .close-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--fg-secondary);
    font-size: 1.2rem;
    line-height: 1;
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .close-btn:hover {
    background: var(--bg-tertiary);
    color: var(--fg);
  }

  .modal-body {
    flex: 1;
    overflow: auto;
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .label {
    font-size: 0.75rem;
    color: var(--fg-secondary);
    font-weight: 500;
  }

  .input {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg);
    color: var(--fg);
    font-family: inherit;
    font-size: 0.875rem;
    transition: border-color var(--transition);
  }

  .input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .content-area {
    min-height: 200px;
    resize: vertical;
    line-height: 1.5;
    font-family: var(--font-sans);
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    border-top: 1px solid var(--border);
  }

  .hint {
    font-size: 0.7rem;
    color: var(--fg-tertiary);
  }

  .btns {
    display: flex;
    gap: var(--space-2);
  }
</style>
