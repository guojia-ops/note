# DeskNote 真实感拟物 UI 优化方案

> 基于视觉设计规范，对比当前实现，制定分阶段优化计划。
> 核心目标：从「莫兰迪低饱和扁平」升级为「真实纸质拟物风格」。

---

## 一、现状分析与差距总览

### 1.1 色板差距

| 颜色 | 当前底色 | 目标底色 | 当前 deep | 目标胶条色 | 差距 |
|------|----------|----------|-----------|-----------|------|
| yellow | `#f3e9b8` | `#FFF4C2` | `#d4b86a` | `#F0D97D` | 底色偏灰暗，需提亮；deep 偏深，需调为胶条色 |
| pink | `#e8c5c8` | `#FFE6E6` | `#c08a90` | `#F5C2C2` | 同上 |
| green | `#c5d6b3` | `#E8F5E0` | `#8fa37a` | `#BED9A8` | 同上 |
| blue | `#b8c9d6` | `#E3F0FC` | `#7e95a8` | `#A9CBEE` | 同上 |
| purple | `#cfc3d6` | `#F1E8F7` | `#9b8aa3` | `#D4BEE4` | 同上 |
| orange→cream | `#e8c9a8` | `#FBF9F3` | `#b8956a` | `#E8E2D3` | **需替换 orange 为 cream（奶白便签）** |

### 1.2 缺失层级（当前 → 目标）

| 层级 | 当前实现 | 目标实现 | 影响范围 |
|------|---------|---------|---------|
| 便签标题文字色 | 统一 `var(--fg)` | 每色独立标题色（如 `#5A4A1F`） | NoteApp / NoteCard |
| 便签正文文字色 | 统一 `var(--fg)` | 每色独立正文色（如 `#6B5A2B`） | NoteApp |
| 顶部胶条色 | 白色渐变叠加 `rgba(255,255,255,0.35)` | 每色独立胶条色（如 `#F0D97D`） | NoteApp / NoteCard |
| 卷角背面色 | 通用 `rgba(0,0,0,0.08)` | 每色独立卷角色（如 `#E8D27A`） | NoteApp |
| 纸张肌理 | 已移除（feTurbulence 性能问题） | 极淡 CSS 噪点（≤5%透明度） | NoteApp / NoteCard |

### 1.3 全局中性色差距

| 变量 | 当前值 | 目标值 | 说明 |
|------|--------|--------|------|
| 页面背景 | `--bg: #ffffff` | `--bg-page: #F5F3EE` | 暖米灰桌面基底 |
| 卡片底色 | 复用 `--bg` | `--bg-card: #FFFFFF` | 白色浮起 |
| 一级边框 | `--border: #e0e0e0` | `--border-primary: #E8E5DD` | 暖灰棕，替代冷灰 |
| 二级边框/hover | `--bg-tertiary: #eeeeee` | `--border-secondary: #F0EDE5` | 更浅暖色 |
| 一级文字 | `--fg: #1a1a1a` | `--text-primary: #2D2D2D` | 深炭灰替代纯黑 |
| 次级文字 | `--fg-secondary: #666666` | `--text-secondary: #6B6B6B` | 微调 |
| 弱化文字 | `--fg-tertiary: #999999` | `--text-muted: #9A9A9A` | 微调 |
| 品牌主色 | `--accent: #f9a825` | `--primary: #F5A623` | 暖橙统一 |
| 主按钮 hover | `filter: brightness(1.05)` | `--primary-hover: #FFB74D` | 明确色值 |
| 主按钮 active | `transform: translateY(1px)` | `--primary-active: #E89512` | 加深凹陷感 |
| 输入框底色 | `--bg` (#ffffff) | `--bg-input: #FBF9F3` | 米白协调 |

### 1.4 阴影差距

| 变量 | 当前值 | 目标值 |
|------|--------|--------|
| 卡片近景阴影 | `--shadow: 0 2px 8px rgba(0,0,0,0.08)` | `--shadow-card: 0 2px 6px rgba(0,0,0,0.06)` |
| 悬浮便签投影 | `--shadow-paper: 0 1px 2px rgba(0,0,0,0.10), 0 8px 24px rgba(0,0,0,0.08)` | `--shadow-float: 0 8px 24px rgba(0,0,0,0.12), 0 2px 6px rgba(0,0,0,0.08)` |
| 强阴影 | `--shadow-strong: 0 4px 16px rgba(0,0,0,0.12)` | 保留，用于右键菜单/Toast |
| 拖动抬升 | `--shadow-paper-lifted: 0 4px 8px rgba(0,0,0,0.12), 0 16px 40px rgba(0,0,0,0.12)` | 保留，语义正确 |

### 1.5 卷角效果差距

| 方面 | 当前实现 | 目标实现 |
|------|---------|---------|
| 实现方式 | `::after` 线性渐变模拟阴影 | `::after` 三角形裁剪 + 独立背面色 + 底部淡阴影 |
| 背面颜色 | 无（纯黑色渐变） | 每色独立卷角背面色（如 `#E8D27A`） |
| 立体感 | 弱（仅明暗渐变） | 强（透出背面纸张色 + 翘起阴影） |

---

## 二、优化方案（分 4 个阶段）

### 阶段 1：色板升级（app.css + types/note.ts）

**影响文件**：`src/app.css`、`src/types/note.ts`、`src-tauri/src/types.rs`

#### 1.1 orange → cream 替换

将 `orange` 颜色枚举替换为 `cream`（奶白便签），涉及：
- `NoteColor` 类型：`'orange'` → `'cream'`
- `NOTE_COLORS` 数组：`'orange'` → `'cream'`
- Rust `NoteColor` 枚举：`Orange` → `Cream`，serde rename `lowercase` 自动处理
- 数据迁移：旧 data.json 中 `"color": "orange"` 需兼容处理（serde rename 或迁移脚本）

#### 1.2 6 色板 CSS 变量升级

每色从 2 个变量（`-bg` + `-deep`）扩展为 5 个变量（`-bg` + `-tape` + `-title` + `-content` + `-fold`）：

```css
:root {
  /* 经典鹅黄（默认） */
  --color-yellow:        #FFF4C2;  /* 纸张底色 */
  --color-yellow-tape:   #F0D97D;  /* 顶部胶条色 */
  --color-yellow-title:  #5A4A1F;  /* 标题文字色 */
  --color-yellow-content:#6B5A2B;  /* 正文文字色 */
  --color-yellow-fold:   #E8D27A;  /* 卷角背面色 */

  /* 樱花粉 */
  --color-pink:          #FFE6E6;
  --color-pink-tape:     #F5C2C2;
  --color-pink-title:    #5C2B2B;
  --color-pink-content:  #703A3A;
  --color-pink-fold:     #EFC0C0;

  /* 薄荷绿 */
  --color-green:         #E8F5E0;
  --color-green-tape:    #BED9A8;
  --color-green-title:   #2F4A23;
  --color-green-content: #3E5C30;
  --color-green-fold:    #B8D4A3;

  /* 晴空蓝 */
  --color-blue:          #E3F0FC;
  --color-blue-tape:     #A9CBEE;
  --color-blue-title:    #1F3D5C;
  --color-blue-content:  #2C4E73;
  --color-blue-fold:     #A0C4E8;

  /* 芋泥紫 */
  --color-purple:        #F1E8F7;
  --color-purple-tape:   #D4BEE4;
  --color-purple-title:  #432C5A;
  --color-purple-content:#543B6E;
  --color-purple-fold:   #CCB5DD;

  /* 奶白便签（替代 orange） */
  --color-cream:         #FBF9F3;
  --color-cream-tape:    #E8E2D3;
  --color-cream-title:   #3A3A3A;
  --color-cream-content: #4F4F4F;
  --color-cream-fold:    #E0D9C9;
}
```

#### 1.3 全局中性色升级

```css
:root,
[data-theme='light'] {
  --bg-page:     #F5F3EE;  /* 页面主背景 */
  --bg-card:     #FFFFFF;  /* 卡片底色 */
  --bg-input:    #FBF9F3;  /* 输入框底色 */
  --bg-secondary:#F0EDE5;  /* 二级背景 */
  --bg-tertiary: #E8E5DD;  /* 三级背景 / hover */
  --border:      #E8E5DD;  /* 一级边框 */
  --border-strong:#D5D0C5; /* 强调边框 */
  --text-primary:#2D2D2D;  /* 正文 */
  --text-secondary:#6B6B6B;/* 次级文字 */
  --text-muted:  #9A9A9A;  /* 占位符 */
  --primary:     #F5A623;  /* 品牌主色 */
  --primary-hover:#FFB74D; /* 主按钮 hover */
  --primary-active:#E89512;/* 主按钮 active */
  /* 向后兼容别名 */
  --fg: var(--text-primary);
  --fg-secondary: var(--text-secondary);
  --fg-tertiary: var(--text-muted);
  --accent: var(--primary);
  --accent-fg: #FFFFFF;
  --bg: var(--bg-card);
}
```

#### 1.4 阴影系统升级

```css
:root {
  --shadow-card:   0 2px 6px rgba(0,0,0,0.06);           /* 卡片近景 */
  --shadow-float:  0 8px 24px rgba(0,0,0,0.12),           /* 悬浮便签远景 */
                   0 2px 6px rgba(0,0,0,0.08);
  --shadow-strong: 0 4px 16px rgba(0,0,0,0.12);           /* 菜单/Toast */
  --shadow-paper:  var(--shadow-float);                    /* 便签窗口别名 */
  --shadow-paper-lifted: 0 4px 8px rgba(0,0,0,0.12),
                         0 16px 40px rgba(0,0,0,0.12);    /* 拖动抬升 */
}
```

---

### 阶段 2：便签窗口拟物化（NoteApp.svelte）

**影响文件**：`src/views/NoteApp.svelte`

#### 2.1 便签容器：多层级色彩引用

**当前**：仅引用 `--note-bg` 和 `--note-border`（2 个变量）

**目标**：引用 5 个变量，通过 JS 动态设置 CSS 自定义属性

```svelte
<script>
  $: noteBg     = note ? `var(--color-${note.color})` : '';
  $: noteTape   = note ? `var(--color-${note.color}-tape)` : '';
  $: noteTitle  = note ? `var(--color-${note.color}-title)` : '';
  $: noteContent= note ? `var(--color-${note.color}-content)` : '';
  $: noteFold   = note ? `var(--color-${note.color}-fold)` : '';
</script>

<main
  style="--note-bg:{noteBg}; --note-tape:{noteTape};
         --note-title:{noteTitle}; --note-content:{noteContent};
         --note-fold:{noteFold}; --rotation:{rotation}deg;"
>
```

#### 2.2 顶部胶条：从白色渐变升级为同色系胶条

**当前**：
```css
.handle {
  background: linear-gradient(180deg,
    rgba(255,255,255,0.35) 0%,
    rgba(255,255,255,0.12) 60%,
    transparent 100%);
}
```

**目标**：
```css
.handle {
  background: linear-gradient(180deg,
    var(--note-tape) 0%,
    color-mix(in srgb, var(--note-tape) 60%, transparent) 60%,
    transparent 100%);
  border-bottom: 1px solid color-mix(in srgb, var(--note-tape) 50%, transparent);
}
```

> 备选方案（兼容性）：若 `color-mix` 不可用，用 `rgba` 半透明叠加：
> ```css
> background: linear-gradient(180deg,
>   var(--note-tape) 0%,
>   rgba(255,255,255,0) 100%);
> opacity: 0.6;
> ```

#### 2.3 卷角效果：从阴影渐变升级为背面透出

**当前**：
```css
.note-app::after {
  background: linear-gradient(135deg,
    transparent 50%,
    rgba(0,0,0,0.08) 50%,
    rgba(0,0,0,0.04) 100%);
}
```

**目标**：
```css
.note-app::after {
  content: '';
  position: absolute;
  right: 0;
  bottom: 0;
  width: 20px;
  height: 20px;
  /* 三角形裁剪：右下角翘起 */
  clip-path: polygon(100% 0, 100% 100%, 0 100%);
  background: var(--note-fold);
  /* 翘起阴影：左上方淡阴影模拟纸张抬起 */
  box-shadow: -2px -2px 4px rgba(0,0,0,0.08);
  border-bottom-right-radius: var(--radius-lg);
  pointer-events: none;
  z-index: 1;
}

/* 卷角下方额外阴影层 */
.note-app::before {
  content: '';
  position: absolute;
  right: 0;
  bottom: 0;
  width: 22px;
  height: 22px;
  background: radial-gradient(circle at 100% 100%,
    rgba(0,0,0,0.08) 0%,
    transparent 70%);
  pointer-events: none;
  z-index: 0;
}
```

#### 2.4 文字颜色：从统一 fg 升级为便签专属色

**当前**：
```css
.title-input { color: var(--fg); }
.content { color: var(--fg); }
```

**目标**：
```css
.title-input { color: var(--note-title); }
.content { color: var(--note-content); }
.title-input::placeholder { color: var(--note-title); opacity: 0.4; }
.content::placeholder { color: var(--note-content); opacity: 0.4; }
```

#### 2.5 纸张肌理：CSS 噪点替代 feTurbulence

> 注意：feTurbulence 已因拖动性能问题移除，改用 CSS `background-image` 叠加噪点 PNG 或
> 极轻量的 `repeating-linear-gradient` 模拟纸纹，透明度 ≤5%。

```css
.note-app {
  background:
    var(--note-bg)
    /* 极淡交叉线纹模拟纸张纤维，替代 feTurbulence */
    repeating-linear-gradient(
      0deg,
      transparent 0px,
      transparent 2px,
      rgba(180,170,150,0.03) 2px,
      rgba(180,170,150,0.03) 3px
    ),
    repeating-linear-gradient(
      90deg,
      transparent 0px,
      transparent 2px,
      rgba(180,170,150,0.03) 2px,
      rgba(180,170,150,0.03) 3px
    );
}
```

> 此方案纯 CSS 无 JS，无 GPU 滤镜，性能安全。透明度 3% 远看均匀、近看有肌理。

#### 2.6 顶部高光线

```css
.note-app {
  /* 顶部 1px 受光高光，模拟纸张受光面 */
  box-shadow:
    inset 0 1px 0 rgba(255,255,255,0.3),
    var(--shadow-float);
}
```

---

### 阶段 3：主窗口卡片拟物化（NoteCard.svelte + MainApp.svelte）

**影响文件**：`src/components/NoteCard.svelte`、`src/views/MainApp.svelte`

#### 3.1 卡片底色与边框

**当前**：`background: var(--bg)` (#ffffff) + `border: 1px solid var(--border)`

**目标**：
```css
.card {
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md); /* 6px，保持不变 */
  box-shadow: var(--shadow-card); /* 新增默认近景阴影 */
}
```

#### 3.2 卡片色条升级为胶条质感

**当前**：6px 色条 + 2px deep 下边线

**目标**：加宽为 8px 胶条，叠加同色系渐变
```css
.color-bar {
  height: 8px;
  background: linear-gradient(180deg,
    var(--card-color-tape) 0%,
    color-mix(in srgb, var(--card-color-tape) 80%, var(--card-color)) 100%);
  border-bottom: 1px solid color-mix(in srgb, var(--card-color-tape) 60%, transparent);
  flex-shrink: 0;
}
```

> 卡片需新增 `--card-color-tape` 变量传入。

#### 3.3 卡片 hover 微立体反馈

**当前**：`box-shadow: var(--shadow-strong); transform: translateY(-2px);`

**目标**：阴影分层 + 抬起幅度微调
```css
.card:hover,
.card:focus-visible {
  box-shadow:
    0 4px 12px rgba(0,0,0,0.08),
    0 1px 3px rgba(0,0,0,0.04);
  transform: translateY(-3px);
  border-color: var(--border-strong);
}
```

#### 3.4 卡片标题色跟随便签色

**当前**：`color: var(--fg)`

**目标**：
```css
.title { color: var(--card-color-title); }
```

> 卡片需新增 `--card-color-title` 变量传入。

#### 3.5 主窗口页面背景

**当前**：`body { background-color: var(--bg); }` (#ffffff)

**目标**：
```css
body { background-color: var(--bg-page); } /* #F5F3EE 暖米灰 */
```

#### 3.6 主按钮微立体

**当前**：`background: var(--accent); hover: filter: brightness(1.05);`

**目标**：
```css
button.primary {
  background: linear-gradient(180deg, var(--primary-hover) 0%, var(--primary) 100%);
  color: #FFFFFF;
  border-color: var(--primary);
  box-shadow: 0 1px 2px rgba(0,0,0,0.08);
}
button.primary:hover {
  background: linear-gradient(180deg, #FFC966 0%, var(--primary-hover) 100%);
  box-shadow: 0 2px 4px rgba(0,0,0,0.10);
}
button.primary:active {
  background: var(--primary-active);
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.12);
  transform: translateY(1px);
}
```

#### 3.7 输入框底色

**当前**：`background: var(--bg)`

**目标**：`background: var(--bg-input);` /* #FBF9F3 米白 */

---

### 阶段 4：深色主题适配

**影响文件**：`src/app.css`

深色主题下便签颜色保持不变（便签本身是浅色纸张），仅调整全局中性色：

```css
[data-theme='dark'] {
  --bg-page:     #2A2825;  /* 深暖灰桌面 */
  --bg-card:     #32302C;  /* 深色卡片 */
  --bg-input:    #383530;  /* 深色输入框 */
  --bg-secondary:#383530;
  --bg-tertiary: #403D38;
  --border:      #4A4640;
  --border-strong:#5A5550;
  --text-primary:#E8E5DD;
  --text-secondary:#AEAAA0;
  --text-muted:  #7A766E;
  --primary:     #F5A623;
  --primary-hover:#FFB74D;
  --primary-active:#E89512;
  /* 便签色板不变，保持纸质观感 */
}
```

---

## 三、变量映射关系（完整对照）

### 便签色 → CSS 变量动态拼接规则

| 用途 | 变量名模式 | 示例（yellow） |
|------|-----------|---------------|
| 纸张底色 | `--color-{color}` | `--color-yellow: #FFF4C2` |
| 顶部胶条 | `--color-{color}-tape` | `--color-yellow-tape: #F0D97D` |
| 标题文字 | `--color-{color}-title` | `--color-yellow-title: #5A4A1F` |
| 正文文字 | `--color-{color}-content` | `--color-yellow-content: #6B5A2B` |
| 卷角背面 | `--color-{color}-fold` | `--color-yellow-fold: #E8D27A` |

> 旧 `-deep` 变量由 `-tape` 替代，需同步更新所有引用点。

### 向后兼容别名

为避免一次性改完所有引用点，在 `:root` 中添加别名过渡：

```css
:root {
  /* 旧变量别名（过渡期保留，后续移除） */
  --color-yellow-deep: var(--color-yellow-tape);
  --color-pink-deep: var(--color-pink-tape);
  --color-green-deep: var(--color-green-tape);
  --color-blue-deep: var(--color-blue-tape);
  --color-purple-deep: var(--color-purple-tape);
  --color-cream-deep: var(--color-cream-tape);

  /* 全局变量别名 */
  --fg: var(--text-primary);
  --fg-secondary: var(--text-secondary);
  --fg-tertiary: var(--text-muted);
  --accent: var(--primary);
  --bg: var(--bg-card);
}
```

---

## 四、数据迁移：orange → cream

### 4.1 Rust 侧（types.rs）

```rust
pub enum NoteColor {
    #[default]
    Yellow,
    Pink,
    Green,
    Blue,
    Purple,
    Cream,  // 原 Orange，serde rename "cream"
}
```

### 4.2 数据兼容

旧 data.json / config.json 中 `"color": "orange"` 需迁移为 `"cream"`。

**方案 A（推荐）**：Rust serde 自定义反序列化，将 `"orange"` 映射为 `Cream`：

```rust
// 在 NoteColor 上实现自定义 deserializer
fn deserialize_note_color<'de, D>(deserializer: D) -> Result<NoteColor, D::Error>
where D: serde::Deserializer<'de> {
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "yellow" => Ok(NoteColor::Yellow),
        "pink" => Ok(NoteColor::Pink),
        "green" => Ok(NoteColor::Green),
        "blue" => Ok(NoteColor::Blue),
        "purple" => Ok(NoteColor::Purple),
        "cream" | "orange" => Ok(NoteColor::Cream),  // 兼容旧数据
        _ => Ok(NoteColor::default()),
    }
}
```

**方案 B**：启动时检测并迁移 data.json 中的 `"orange"` → `"cream"`（一次性脚本）。

---

## 五、实施清单

### 阶段 1：色板升级
- [ ] app.css：替换 6 色板 CSS 变量（2→5 个/色）
- [ ] app.css：升级全局中性色变量（暖灰系）
- [ ] app.css：升级阴影系统变量
- [ ] app.css：添加向后兼容别名
- [ ] types/note.ts：`orange` → `cream`
- [ ] types.rs：`Orange` → `Cream` + serde 兼容
- [ ] 全局 `grep -r "orange"` 确认无残留引用

### 阶段 2：便签窗口拟物化
- [ ] NoteApp.svelte：JS 动态传入 5 个 CSS 变量
- [ ] NoteApp.svelte：胶条 `--note-tape` 替换白色渐变
- [ ] NoteApp.svelte：卷角 `clip-path` + `--note-fold` 背面透出
- [ ] NoteApp.svelte：标题/正文文字色跟随便签色
- [ ] NoteApp.svelte：CSS 噪点肌理叠加（≤3% 透明度）
- [ ] NoteApp.svelte：顶部 `inset` 高光线
- [ ] 拖动性能验证（确保无 GPU 重绘瓶颈）

### 阶段 3：主窗口卡片拟物化
- [ ] NoteCard.svelte：传入 `--card-color-tape` / `--card-color-title`
- [ ] NoteCard.svelte：色条升级为胶条渐变
- [ ] NoteCard.svelte：卡片默认阴影 + hover 分层阴影
- [ ] NoteCard.svelte：标题色跟随便签色
- [ ] MainApp.svelte：页面背景 `--bg-page`
- [ ] app.css：主按钮微立体渐变 + 按压凹陷
- [ ] app.css：输入框 `--bg-input` 米白底色

### 阶段 4：深色主题适配
- [ ] app.css：深色主题全局变量（暖灰系）
- [ ] 验证便签色板在深色背景下可读性
- [ ] 验证卡片/菜单/Toast 在深色主题下样式

---

## 六、性能约束

| 指标 | 约束 | 说明 |
|------|------|------|
| CSS 噪点透明度 | ≤ 3% | 纯 CSS repeating-linear-gradient，无 SVG 滤镜 |
| 卷角伪元素 | ≤ 2 个 | `::after` 背面 + `::before` 阴影，无额外 DOM |
| 拖动期间 transition | `none` | 保持现有 `.dragging { transition: none }` |
| color-mix() | 可选 | 若 WebView2 不支持则降级为 rgba 叠加 |
| 新增 CSS 变量 | 0 额外 HTTP | 纯 CSS 变量，无外部资源加载 |

---

## 七、验收标准

1. **视觉一致性**：便签窗口与卡片墙的同一颜色视觉一致
2. **拟物真实感**：卷角可见背面色、胶条有同色系加深、文字在便签底色上清晰可读
3. **拖动流畅度**：拖动便签无卡顿（与优化前持平）
4. **深色主题**：便签保持浅色纸质观感，主界面深暖灰协调
5. **数据兼容**：旧 data.json 的 `orange` 便签自动迁移为 `cream`
6. **验证命令**：`pnpm check` 0 errors/warnings + `cargo test` 全部通过
