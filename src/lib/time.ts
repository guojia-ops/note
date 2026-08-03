// 相对时间工具（SOP 6.4：如「3 分钟前」）

/** 将时间戳（毫秒）转为相对时间字符串 */
export function relativeTime(ts: number): string {
  const now = Date.now();
  const diff = Math.max(0, now - ts);
  const sec = Math.floor(diff / 1000);

  if (sec < 60) return '刚刚';
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min} 分钟前`;
  const hour = Math.floor(min / 60);
  if (hour < 24) return `${hour} 小时前`;
  const day = Math.floor(hour / 24);
  if (day < 7) return `${day} 天前`;
  // 超过 7 天显示日期
  const d = new Date(ts);
  const m = d.getMonth() + 1;
  const dd = d.getDate();
  if (d.getFullYear() === new Date().getFullYear()) {
    return `${m}月${dd}日`;
  }
  return `${d.getFullYear()}/${m}/${dd}`;
}
