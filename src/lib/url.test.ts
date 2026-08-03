import { describe, it, expect } from 'vitest';
import { splitByUrl, normalizeUrl, getPreviewText } from './url';

describe('splitByUrl', () => {
  it('空字符串返回空数组', () => {
    expect(splitByUrl('')).toEqual([]);
  });

  it('无 URL 的纯文本返回单个 text 段', () => {
    const segs = splitByUrl('你好世界 hello world');
    expect(segs).toHaveLength(1);
    expect(segs[0]).toEqual({ type: 'text', content: '你好世界 hello world' });
  });

  it('识别 http URL', () => {
    const segs = splitByUrl('访问 http://example.com 看看');
    expect(segs).toHaveLength(3);
    expect(segs[0]).toEqual({ type: 'text', content: '访问 ' });
    expect(segs[1]).toEqual({ type: 'url', content: 'http://example.com' });
    expect(segs[2]).toEqual({ type: 'text', content: ' 看看' });
  });

  it('识别 https URL', () => {
    const segs = splitByUrl('https://github.com');
    expect(segs).toHaveLength(1);
    expect(segs[0]).toEqual({ type: 'url', content: 'https://github.com' });
  });

  it('识别 www 开头的裸域名', () => {
    const segs = splitByUrl('去 www.google.com 搜索');
    expect(segs[1]).toEqual({ type: 'url', content: 'www.google.com' });
  });

  it('识别 ftp 协议', () => {
    const segs = splitByUrl('ftp://files.example.com');
    expect(segs[0]).toEqual({ type: 'url', content: 'ftp://files.example.com' });
  });

  it('URL 在末尾时无尾部空文本段', () => {
    const segs = splitByUrl('前置文本 https://example.com');
    expect(segs).toHaveLength(2);
    expect(segs[1]).toEqual({ type: 'url', content: 'https://example.com' });
  });

  it('URL 在开头时无前导空文本段', () => {
    const segs = splitByUrl('https://example.com 后置文本');
    expect(segs).toHaveLength(2);
    expect(segs[0]).toEqual({ type: 'url', content: 'https://example.com' });
  });

  it('多个 URL 混合文本', () => {
    const segs = splitByUrl('a http://x.com b www.y.com c');
    expect(segs).toHaveLength(5);
    expect(segs.map((s) => s.type)).toEqual(['text', 'url', 'text', 'url', 'text']);
    expect(segs[1].content).toBe('http://x.com');
    expect(segs[3].content).toBe('www.y.com');
  });

  it('不识别纯协议前缀', () => {
    // http:// 后无内容不应匹配（正则要求至少一个非空白字符）
    const segs = splitByUrl('http://');
    expect(segs).toHaveLength(1);
    expect(segs[0].type).toBe('text');
  });
});

describe('normalizeUrl', () => {
  it('http 协议保持不变', () => {
    expect(normalizeUrl('http://example.com')).toBe('http://example.com');
  });

  it('https 协议保持不变', () => {
    expect(normalizeUrl('https://example.com')).toBe('https://example.com');
  });

  it('ftp 协议保持不变', () => {
    expect(normalizeUrl('ftp://files.example.com')).toBe('ftp://files.example.com');
  });

  it('www 开头补 https://', () => {
    expect(normalizeUrl('www.example.com')).toBe('https://www.example.com');
  });

  it('WWW 大写也补 https://', () => {
    expect(normalizeUrl('WWW.EXAMPLE.COM')).toBe('https://WWW.EXAMPLE.COM');
  });

  it('非 URL 文本原样返回', () => {
    expect(normalizeUrl('普通文本')).toBe('普通文本');
  });
});

describe('getPreviewText', () => {
  it('有标题时返回标题', () => {
    expect(getPreviewText({ title: '我的标题', content: '内容' })).toBe('我的标题');
  });

  it('标题前后空白被 trim', () => {
    expect(getPreviewText({ title: '  标题  ', content: '' })).toBe('标题');
  });

  it('空标题空内容返回占位', () => {
    expect(getPreviewText({ title: '', content: '' })).toBe('（空便签）');
  });

  it('空标题有内容返回内容前 30 字', () => {
    const content = '一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十';
    expect(getPreviewText({ title: '', content })).toBe(content);
  });

  it('超长内容截断并加省略号', () => {
    const content = '一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五';
    // 35 字 > 30
    const preview = getPreviewText({ title: '', content });
    expect(preview.length).toBe(31); // 30 字 + 省略号
    expect(preview.endsWith('…')).toBe(true);
  });

  it('内容中 CRLF 折叠为单空格', () => {
    expect(getPreviewText({ title: '', content: '行1\r\n行2' })).toBe('行1 行2');
  });
});
