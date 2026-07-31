// URL 自动识别（PRD 5：纯文本 + URL 自动识别）
// 用于在便签正文中将 URL 转为可点击链接

/** URL 匹配正则
 * 覆盖 http/https/ftp 协议，以及 www. 开头的裸域名
 * 不过度匹配：要求 URL 前后是空白或字符串边界
 */
const URL_REGEX = /(?:https?|ftp):\/\/[^\s<>"']+|www\.[^\s<>"']+/gi;

export interface UrlSegment {
  type: 'text' | 'url';
  content: string;
}

/** 将文本按 URL 拆分为段
 * 前端渲染时遍历 segments，url 段渲染为 <a>
 */
export function splitByUrl(text: string): UrlSegment[] {
  if (!text) return [];

  const segments: UrlSegment[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  // 重置 regex（全局标志有 lastIndex 状态）
  URL_REGEX.lastIndex = 0;

  while ((match = URL_REGEX.exec(text)) !== null) {
    // 前面的文本段
    if (match.index > lastIndex) {
      segments.push({ type: 'text', content: text.slice(lastIndex, match.index) });
    }
    // URL 段
    segments.push({ type: 'url', content: match[0] });
    lastIndex = match.index + match[0].length;
  }

  // 尾部文本段
  if (lastIndex < text.length) {
    segments.push({ type: 'text', content: text.slice(lastIndex) });
  }

  return segments;
}

/** 给 URL 补全协议（用于 <a href>）
 * www.example.com → https://www.example.com
 */
export function normalizeUrl(url: string): string {
  if (url.startsWith('http://') || url.startsWith('https://') || url.startsWith('ftp://')) {
    return url;
  }
  if (url.toLowerCase().startsWith('www.')) {
    return `https://${url}`;
  }
  return url;
}

/** 获取便签列表预览文本（PRD 9.2：标题为空时用正文前 30 字预览） */
export function getPreviewText(note: { title: string; content: string }): string {
  if (note.title.trim()) return note.title.trim();
  const content = note.content.trim();
  if (!content) return '（空便签）';
  // 按空白折叠，最多 30 字
  const folded = content.replace(/\s+/g, ' ');
  return folded.length > 30 ? `${folded.slice(0, 30)}…` : folded;
}
