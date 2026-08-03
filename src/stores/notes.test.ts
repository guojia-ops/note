import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import type { Note } from '../types/note';

// Mock @tauri-apps/api/core 的 invoke
// notes store 通过 getNotes → invoke('get_notes') 拉数据
const invokeMock = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => invokeMock(cmd, args),
}));

// Mock @tauri-apps/api/event 的 listen（subscribeAllNotes 会调用）
const listenMock = vi.fn();
vi.mock('@tauri-apps/api/event', () => ({
  listen: (event: string, cb: (e: { payload: unknown }) => void) => {
    listenMock(event, cb);
    // 返回 unlisten 函数
    return Promise.resolve(() => {});
  },
}));

// 导入被测模块（在 mock 之后）
import { notes, notesLoading, refreshNotes } from './notes';

// 构造测试便签数据
function makeNote(overrides: Partial<Note> = {}): Note {
  return {
    id: 'n1',
    title: '',
    content: '',
    color: 'yellow',
    width: 240,
    height: 240,
    x: 100,
    y: 100,
    created_at: 1000,
    updated_at: 1000,
    pinned: false,
    ...overrides,
  };
}

describe('notes store', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
    notes.set([]);
    notesLoading.set(true);
  });

  it('refreshNotes 从后端拉取便签并按 updated_at 倒序', async () => {
    const n1 = makeNote({ id: '1', updated_at: 1000 });
    const n2 = makeNote({ id: '2', updated_at: 3000 });
    const n3 = makeNote({ id: '3', updated_at: 2000 });
    invokeMock.mockResolvedValue([n1, n2, n3]);

    await refreshNotes();

    const list = get(notes);
    expect(list).toHaveLength(3);
    // 倒序：最新的在前
    expect(list[0].id).toBe('2'); // updated_at=3000
    expect(list[1].id).toBe('3'); // updated_at=2000
    expect(list[2].id).toBe('1'); // updated_at=1000
  });

  it('refreshNotes 完成后 notesLoading 置为 false', async () => {
    invokeMock.mockResolvedValue([]);
    await refreshNotes();
    expect(get(notesLoading)).toBe(false);
  });

  it('refreshNotes 抛异常时 notesLoading 也置为 false', async () => {
    invokeMock.mockRejectedValue(new Error('网络错误'));
    await expect(refreshNotes()).rejects.toThrow('网络错误');
    expect(get(notesLoading)).toBe(false);
  });

  it('refreshNotes 空列表正常处理', async () => {
    invokeMock.mockResolvedValue([]);
    await refreshNotes();
    expect(get(notes)).toEqual([]);
  });

  it('refreshNotes 调用 invoke 的命令名正确', async () => {
    invokeMock.mockResolvedValue([]);
    await refreshNotes();
    expect(invokeMock).toHaveBeenCalledWith('get_notes', undefined);
  });

  it('refreshNotes 相同 updated_at 时顺序稳定（保持原顺序）', async () => {
    const n1 = makeNote({ id: '1', updated_at: 1000 });
    const n2 = makeNote({ id: '2', updated_at: 1000 });
    invokeMock.mockResolvedValue([n1, n2]);
    await refreshNotes();
    const list = get(notes);
    // sort 稳定：相同 key 保持原顺序
    expect(list[0].id).toBe('1');
    expect(list[1].id).toBe('2');
  });
});
