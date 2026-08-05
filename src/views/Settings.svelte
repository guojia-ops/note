<script lang="ts">
  // 设置窗口（SOP 7.3-7.13）
  // 8 项设置：开机自启、全局快捷键、默认颜色、数据目录、自动备份+份数、主题、导出/导入、关闭行为
  import { config, applyTheme } from '../stores/config';
  import {
    updateConfig,
    exportNotes,
    importNotes,
    getDataDir,
    registerHotkey,
    setAutoStart,
  } from '../lib/commands';
  import { NOTE_COLORS, type Config, type NoteColor, type Theme, type CloseAction } from '../types/note';

  // 本地状态：输入中的值，失焦或保存时提交（防抖避免每按一次都发请求）
  // 开关/单选/颜色：直接提交
  // 文本输入/数字：失焦或回车时提交

  const THEME_OPTIONS: { value: Theme; label: string }[] = [
    { value: 'light', label: '浅色' },
    { value: 'dark', label: '深色' },
    { value: 'system', label: '跟随系统' },
  ];

  const CLOSE_OPTIONS: { value: CloseAction; label: string; desc: string }[] = [
    { value: 'tray', label: '最小化到托盘', desc: '主窗口关闭后仍在托盘运行，便签继续显示' },
    { value: 'quit', label: '完全退出', desc: '关闭主窗口即退出整个应用' },
  ];

  // 数据目录（只读显示）
  let dataDir = '';
  (async () => {
    try {
      dataDir = await getDataDir();
    } catch {
      dataDir = '(读取失败)';
    }
  })();

  // 热键输入
  let hotkeyInput = $config.hotkey;
  $: if (!$config) { /* noop */ } // 订阅 config store
  function syncHotkey() {
    hotkeyInput = $config.hotkey;
  }
  $: $config && syncHotkey();

  let hotkeySaving = false;
  async function saveHotkey() {
    const v = hotkeyInput.trim() || $config.hotkey;
    if (v === $config.hotkey) return;
    try {
      hotkeySaving = true;
      await updateConfig({ hotkey: v });
      // SOP 8.9：快捷键变更后重新注册（Rust 侧先 unregister_all 再 register）
      await registerHotkey();
    } catch (e) {
      console.error('[DeskNote] 快捷键注册失败', e);
    } finally {
      hotkeySaving = false;
    }
  }

  // backup_keep 数字
  let backupKeepInput = String($config.backup_keep);
  function syncBackupKeep() {
    backupKeepInput = String($config.backup_keep);
  }
  $: $config && syncBackupKeep();

  let backupKeepSaving = false;
  async function saveBackupKeep() {
    const n = parseInt(backupKeepInput, 10);
    const v = isNaN(n) ? 5 : Math.max(1, Math.min(20, n));
    if (v === $config.backup_keep) return;
    try {
      backupKeepSaving = true;
      await updateConfig({ backup_keep: v });
    } finally {
      backupKeepSaving = false;
    }
  }

  // 开关 / 直接提交型
  let savingSwitch = new Set<string>();
  async function toggleSwitch<K extends keyof Pick<Config, 'auto_backup' | 'always_on_top'>>(
    key: K,
    value: Config[K],
  ) {
    try {
      savingSwitch.add(key);
      await updateConfig({ [key]: value } as Partial<Config>);
    } finally {
      savingSwitch.delete(key);
    }
  }

  // 开机自启：单独处理，需同时更新 config + 系统注册项（v1.1 优化阶段 3）
  let autoStartSaving = false;
  let autoStartMsg = '';
  async function toggleAutoStart(enabled: boolean) {
    if (enabled === $config.auto_start) return;
    try {
      autoStartSaving = true;
      autoStartMsg = '';
      await setAutoStart(enabled);
      autoStartMsg = enabled ? '已启用开机自启' : '已关闭开机自启';
    } catch (e) {
      // 失败时回退开关状态（config store 未更新，UI 会自动回退）
      autoStartMsg = `设置失败：${e instanceof Error ? e.message : String(e)}`;
    } finally {
      autoStartSaving = false;
      // 3 秒后清空提示
      setTimeout(() => { autoStartMsg = ''; }, 3000);
    }
  }

  // 默认颜色
  let savingColor = false;
  async function setDefaultColor(c: NoteColor) {
    if (c === $config.default_color) return;
    try {
      savingColor = true;
      await updateConfig({ default_color: c });
    } finally {
      savingColor = false;
    }
  }

  // 主题：切换时立即 apply
  let savingTheme = false;
  async function setTheme(t: Theme) {
    if (t === $config.theme) return;
    try {
      savingTheme = true;
      await updateConfig({ theme: t });
      applyTheme(t);
    } finally {
      savingTheme = false;
    }
  }

  // close_action
  let savingClose = false;
  async function setCloseAction(a: CloseAction) {
    if (a === $config.close_action) return;
    try {
      savingClose = true;
      await updateConfig({ close_action: a });
    } finally {
      savingClose = false;
    }
  }

  // 导出
  let exporting = false;
  let exportMsg = '';
  // 将用户输入解析为绝对路径：相对路径拼接到数据目录下
  function resolveExportPath(p: string): string {
    const trimmed = p.trim();
    if (!trimmed) return '';
    // Windows 绝对路径：盘符开头（C:\）或 UNC（\\）
    if (/^[a-zA-Z]:[\\/]/.test(trimmed) || trimmed.startsWith('\\\\')) return trimmed;
    // 相对路径 → 拼接到数据目录
    const sep = dataDir && !dataDir.endsWith('\\') && !dataDir.endsWith('/') ? '\\' : '';
    return dataDir ? `${dataDir}${sep}${trimmed}` : trimmed;
  }
  async function handleExport() {
    const date = new Date().toISOString().slice(0, 10);
    const defaultName = `desknote-backup-${date}.json`;
    const sep = dataDir && !dataDir.endsWith('\\') && !dataDir.endsWith('/') ? '\\' : '';
    const defaultPath = dataDir ? `${dataDir}${sep}${defaultName}` : defaultName;
    const input = window.prompt('输入导出文件路径（.json）', defaultPath);
    if (!input) return;
    const resolved = resolveExportPath(input);
    try {
      exporting = true;
      exportMsg = '';
      await exportNotes(resolved);
      exportMsg = `导出成功：${resolved}`;
    } catch (e) {
      exportMsg = `导出失败：${e instanceof Error ? e.message : String(e)}`;
    } finally {
      exporting = false;
    }
  }

  // 导入（用 prompt 输入路径，与导出对称；Tauri 2.x WebView2 不给 File 注入 path）
  let importing = false;
  let importMsg = '';
  async function handleImport() {
    const sep = dataDir && !dataDir.endsWith('\\') && !dataDir.endsWith('/') ? '\\' : '';
    const defaultPath = dataDir ? `${dataDir}${sep}desknote-backup.json` : 'desknote-backup.json';
    const input = window.prompt('输入要导入的文件路径（.json）', defaultPath);
    if (!input) return;
    const resolved = resolveExportPath(input);
    try {
      importing = true;
      importMsg = '';
      const res = await importNotes(resolved);
      importMsg = `导入成功：新增 ${res.imported} 条，跳过 ${res.skipped} 条`;
    } catch (err) {
      importMsg = `导入失败：${err instanceof Error ? err.message : String(err)}`;
    } finally {
      importing = false;
    }
  }
</script>

<main class="settings">
  <h1>设置</h1>

  <section class="card">
    <h2>通用</h2>

    <!-- 开机自启（v1.1 优化阶段 3：config + 系统注册项同步） -->
    <div class="row">
      <div class="field-info">
        <span class="label">开机自启</span>
        <span class="desc">登录系统后自动启动 DeskNote{#if autoStartMsg}<span class="inline-msg">{autoStartMsg}</span>{/if}</span>
      </div>
      <label class="switch">
        <input
          type="checkbox"
          checked={$config.auto_start}
          disabled={autoStartSaving}
          on:change={(e) =>
            void toggleAutoStart((e.currentTarget as HTMLInputElement).checked)}
        />
        <span class="slider"></span>
      </label>
    </div>

    <!-- 全局快捷键（SOP 7.5，阶段 8 实现实际注册） -->
    <div class="row">
      <div class="field-info">
        <span class="label">全局快捷键</span>
        <span class="desc">任意位置按此键新建便签（例：Ctrl+Alt+N）</span>
        <span class="badge warn">阶段 8 实现</span>
      </div>
      <div class="input-wrap">
        <input
          class="input-inline"
          type="text"
          value={hotkeyInput}
          on:input={(e) => (hotkeyInput = (e.currentTarget as HTMLInputElement).value)}
          on:blur={saveHotkey}
          on:keydown={(e) => {
            if (e.key === 'Enter') {
              (e.currentTarget as HTMLInputElement).blur();
            }
          }}
        />
        {#if hotkeySaving}<span class="saving">保存中…</span>{/if}
      </div>
    </div>

    <!-- 默认颜色（SOP 7.6） -->
    <div class="row">
      <div class="field-info">
        <span class="label">新建便签默认颜色</span>
        <span class="desc">新建便签时使用的颜色</span>
      </div>
      <div class="color-swatches">
        {#each NOTE_COLORS as c}
          <button
            class="swatch"
            class:active={$config.default_color === c}
            style="background: var(--color-{c}); border-color: var(--color-{c}-deep);"
            disabled={savingColor}
            on:click={() => void setDefaultColor(c)}
            title={c}
            aria-label={c}
          ></button>
        {/each}
      </div>
    </div>

    <!-- 便签始终置顶（v1.1 优化阶段 2.1） -->
    <div class="row">
      <div class="field-info">
        <span class="label">便签始终置顶</span>
        <span class="desc">贴出的便签悬浮在所有窗口前方，关闭后可被其他窗口遮挡</span>
      </div>
      <label class="switch">
        <input
          type="checkbox"
          checked={$config.always_on_top ?? true}
          disabled={savingSwitch.has('always_on_top')}
          on:change={(e) =>
            void toggleSwitch('always_on_top', (e.currentTarget as HTMLInputElement).checked)}
        />
        <span class="slider"></span>
      </label>
    </div>

    <!-- 数据目录（SOP 7.7） -->
    <div class="row">
      <div class="field-info">
        <span class="label">数据存储目录</span>
        <span class="desc">便签数据与备份存放位置（只读）</span>
      </div>
      <span class="mono small">{dataDir || '读取中…'}</span>
    </div>
  </section>

  <section class="card">
    <h2>备份</h2>

    <!-- 自动备份开关（SOP 7.8） -->
    <div class="row">
      <div class="field-info">
        <span class="label">自动备份</span>
        <span class="desc">每次写入自动创建时间戳备份</span>
      </div>
      <label class="switch">
        <input
          type="checkbox"
          checked={$config.auto_backup}
          disabled={savingSwitch.has('auto_backup')}
          on:change={(e) => void toggleSwitch('auto_backup', (e.currentTarget as HTMLInputElement).checked)}
        />
        <span class="slider"></span>
      </label>
    </div>

    <!-- 备份保留份数（SOP 7.8） -->
    <div class="row">
      <div class="field-info">
        <span class="label">备份保留份数</span>
        <span class="desc">超过此数的旧备份自动清理（1-20）</span>
      </div>
      <div class="input-wrap">
        <input
          class="input-inline small"
          type="number"
          min="1"
          max="20"
          value={backupKeepInput}
          on:input={(e) => (backupKeepInput = (e.currentTarget as HTMLInputElement).value)}
          on:blur={saveBackupKeep}
        />
        {#if backupKeepSaving}<span class="saving">保存中…</span>{/if}
      </div>
    </div>

    <!-- 导出（SOP 7.10） -->
    <div class="row">
      <div class="field-info">
        <span class="label">导出全部便签</span>
        <span class="desc">另存为 JSON 文件，可在其他设备导入</span>
      </div>
      <button class="primary" disabled={exporting} on:click={handleExport}>
        {exporting ? '导出中…' : '导出…'}
      </button>
    </div>

    <!-- 导入（SOP 7.11） -->
    <div class="row">
      <div class="field-info">
        <span class="label">导入便签</span>
        <span class="desc">输入 JSON 文件路径，根据 id 去重</span>
      </div>
      <button disabled={importing} on:click={handleImport}>
        {importing ? '导入中…' : '导入…'}
      </button>
    </div>

    {#if exportMsg || importMsg}
      <div class="msg-box">
        {#if exportMsg}<p>{exportMsg}</p>{/if}
        {#if importMsg}<p>{importMsg}</p>{/if}
      </div>
    {/if}
  </section>

  <section class="card">
    <h2>外观</h2>

    <!-- 主题（SOP 7.9） -->
    <div class="row">
      <div class="field-info">
        <span class="label">主题</span>
        <span class="desc">应用整体配色（设置窗口立即生效）</span>
      </div>
      <div class="seg">
        {#each THEME_OPTIONS as t}
          <button
            class="seg-btn"
            class:active={$config.theme === t.value}
            disabled={savingTheme}
            on:click={() => void setTheme(t.value)}
          >
            {t.label}
          </button>
        {/each}
      </div>
    </div>

    <!-- 关闭行为（SOP 7.12） -->
    <div class="row stack">
      <div class="field-info">
        <span class="label">关闭主窗口时</span>
        <span class="desc">阶段 8 实现实际拦截（功能待实现）</span>
        <span class="badge warn">阶段 8 实现</span>
      </div>
      <div class="radio-group">
        {#each CLOSE_OPTIONS as opt}
          <label class="radio-card" class:active={$config.close_action === opt.value}>
            <input
              type="radio"
              name="close_action"
              checked={$config.close_action === opt.value}
              disabled={savingClose}
              on:change={() => void setCloseAction(opt.value)}
            />
            <div class="radio-content">
              <div class="radio-title">{opt.label}</div>
              <div class="radio-desc">{opt.desc}</div>
            </div>
          </label>
        {/each}
      </div>
    </div>
  </section>
</main>

<style>
  .settings {
    width: 100%;
    height: 100%;
    overflow: auto;
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  h1 {
    font-size: 1.4rem;
    font-weight: 700;
    color: var(--fg);
    margin-bottom: var(--space-1);
  }

  .card {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  h2 {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--fg);
    padding-bottom: var(--space-2);
    border-bottom: 1px solid var(--border);
    margin: 0 calc(-1 * var(--space-4)) var(--space-1);
    padding-left: var(--space-4);
    padding-right: var(--space-4);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) 0;
  }

  .row.stack {
    flex-direction: column;
    align-items: stretch;
  }

  .field-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .label {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--fg);
  }

  .desc {
    font-size: 0.75rem;
    color: var(--fg-tertiary);
  }

  .inline-msg {
    margin-left: var(--space-2);
    color: var(--accent);
    font-size: 0.72rem;
  }

  .badge {
    display: inline-block;
    width: fit-content;
    font-size: 0.65rem;
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    margin-top: 2px;
  }

  .badge.warn {
    background: var(--accent);
    color: var(--accent-fg);
  }

  .input-wrap {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .input-inline {
    padding: var(--space-1) var(--space-2);
    height: 30px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
    font-family: inherit;
    font-size: 0.85rem;
    min-width: 140px;
  }

  .input-inline:focus {
    outline: none;
    border-color: var(--accent);
  }

  .input-inline.small {
    width: 72px;
    min-width: 0;
  }

  .saving {
    font-size: 0.7rem;
    color: var(--accent);
  }

  .mono {
    font-family: var(--font-mono);
  }

  .small {
    font-size: 0.75rem;
    color: var(--fg-secondary);
    max-width: 240px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Switch */
  .switch {
    position: relative;
    display: inline-block;
    width: 40px;
    height: 22px;
    flex-shrink: 0;
  }

  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    inset: 0;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 22px;
    transition: all var(--transition);
  }

  .slider::before {
    position: absolute;
    content: '';
    height: 16px;
    width: 16px;
    left: 2px;
    top: 50%;
    transform: translateY(-50%);
    background: var(--fg-tertiary);
    border-radius: 50%;
    transition: all var(--transition);
  }

  .switch input:checked + .slider {
    background: var(--accent);
    border-color: var(--accent);
  }

  .switch input:checked + .slider::before {
    transform: translate(18px, -50%);
    background: #fff;
  }

  .switch input:disabled + .slider {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* 颜色色板 */
  .color-swatches {
    display: flex;
    gap: var(--space-2);
  }

  .swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
    padding: 0;
    cursor: pointer;
    transition: all var(--transition);
  }

  .swatch:hover {
    transform: scale(1.1);
  }

  .swatch.active {
    box-shadow: 0 0 0 2px var(--bg), 0 0 0 4px var(--accent);
  }

  /* Segment */
  .seg {
    display: flex;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .seg-btn {
    padding: var(--space-1) var(--space-3);
    height: 32px;
    border: none;
    border-radius: 0;
    background: var(--bg);
    color: var(--fg-secondary);
    font-size: 0.85rem;
    cursor: pointer;
    transition: all var(--transition);
  }

  .seg-btn + .seg-btn {
    border-left: 1px solid var(--border);
  }

  .seg-btn:hover {
    background: var(--bg-tertiary);
  }

  .seg-btn.active {
    background: var(--accent);
    color: var(--accent-fg);
  }

  .msg-box {
    padding: var(--space-2) var(--space-3);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    font-size: 0.8rem;
    color: var(--fg-secondary);
    line-height: 1.6;
    margin-top: var(--space-1);
  }

  /* 关闭行为单选卡片 */
  .radio-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .radio-card {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition);
  }

  .radio-card:hover {
    border-color: var(--border-strong);
    background: var(--bg-secondary);
  }

  .radio-card.active {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 8%, var(--bg));
  }

  .radio-card input {
    margin-top: 4px;
  }

  .radio-card:has(input:disabled) {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .radio-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .radio-title {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--fg);
  }

  .radio-desc {
    font-size: 0.72rem;
    color: var(--fg-tertiary);
    line-height: 1.4;
  }
</style>
