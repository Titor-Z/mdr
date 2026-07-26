# AGENTS.md — mdr (Markdown Renderer)

> 本文档服务于 AI Agent 与人类开发者协作开发，包含项目演进过程中的
> 所有关键信息。每次迭代后请更新此文件。

---

## 1. Changelog

### v0.2.0 (2026-07-26)

**HTTP 服务器 + SSE 热重载 + 表格渲染 + 段落换行 + 任务列表 + VitePress 风格**

#### 新增
- HTTP 服务器模式（`mdr serve --port 8080`）：
  - 首页卡片式文件列表（竖线 + 文件名 + 日期，终端 TUI 同款风格）
  - 详情页左右布局（左侧文件树，右侧渲染内容）
  - comrak → HTML 渲染，syntect 代码语法高亮
  - 暗色/亮色主题切换（CSS 变量，localStorage 持久化）
  - 模板系统（base / index / document，minijinja）
  - 内联 CSS/JS，无需外部资源依赖
- 表格渲染：四周边框（┌─┬─┐│├─┼─┤└─┴─┘）、列宽按视窗比例分配、自动换行、垂直居中
- 段落自动换行（复用 wrap_spans，按视窗宽度折行）
- 列表项自动换行（首行 bullet，后续缩进）
- 任务列表：☐ 未选中 / ☑ 已选中，支持嵌套

**VitePress 风格页面**
- 完整 CSS 变量体系（`--vp-c-*`）：brand / tip / success / warning / danger / caution
- 页面导航（On This Page）：小屏下拉菜单 + 大屏右侧固定栏（断点 1280px）
- 右侧目录导航（toc-sidebar），窄屏可折叠按钮「页面导航 ▾」
- 响应式：< 768px 手机布局、768-1280px 平板、≥ 1280px 桌面三档
- 开发模式 `MDR_DEV=1`：CSS/JS/模板从文件系统加载，改完重启即可无需编译

**VitePress 风格代码块**
- 双层布局：标题栏（语言名）+ 代码区
- 右上角悬浮复制按钮
- 代码组（`::: code-group`）标签页切换，支持 `\`\`js [config.js]` 语法
- 自定义容器：info / tip / warning / danger / caution / important / note / details
  - 支持自定义标题 `::: danger STOP`
  - details 用 `<details><summary>` 实现折叠
- 代码高亮使用 comrak SyntectAdapter 官方插件
- 语言别名映射：ts→js, py→python, sh→bash
- 字体尺寸对齐 VitePress（h1: 28-32px, 正文: 16px, code: 0.875em）

**交互和体验**
- SSE 热重载：`MDR_DEV=1` 时文件监听 + 浏览器自动刷新
  - notify 文件监听 + broadcast 广播通道
  - `/events` SSE 端点（15s keep-alive）
  - JS EventSource 连接，收到 reload 自动刷新页面
  - 500ms 去抖，避免频繁刷新
- 鼠标框选文字（Shift+click 绕过鼠标捕获）
- Ctrl+点击打开链接（检测行内 URL，调用系统浏览器）
- 鼠标捕获开关（`Ctrl+m` 切换）
- 内容区左右 1 列 padding（不触边、不挡滚动条）
- 渲染器预留滚动条空间（全局 viewport - 2）

#### 修复
- 搜索栏消失（Block 边框占 2 行但状态栏只分配 1 行）
- 未选中卡片右偏移 1 列（indent 变量导致）
- 中文表格对齐（改用 unicode-width 计算显示宽度）
- 表格右侧边框被滚动条遮挡（预留列宽）
- 段落和表格右边缘与滚动条重叠（统一 viewport - 2）
- 代码块 pre 内联背景色移除，改由 CSS 控制
- 代码块语言标签偏移（class="language-" 长度 16 误写 17）
- 代码组第二个代码块丢失高亮（wrap_code_blocks 重复包装）
- 自定义容器预处理（trim_start_matches 残留空格导致类型解析错误）

#### 依赖变更
- 新增 `unicode-width = "0.2"`
- 新增 `axum`, `tokio`, `minijinja`, `tower-http`, `serde`

### v0.1.0 (2026-07-26)

**初始版本 — 终端 Markdown 渲染器 + 文件选择器 + 内置 Pager**

#### 新增
- 终端 Markdown 渲染引擎（comrak AST → ratatui 样式）
- 代码块语法高亮（syntect）
- 内置 TUI pager（无需 less/more）
- 文件选择器首页（扫描 CWD 下的 .md 文件）
- 搜索功能（`/` 进入，`n`/`N` 跳转，匹配红色高亮）
- 卡片式文件列表布局
- 鼠标滚轮 / 点击 / 拖拽滚动条
- 帮助抽屉（`?` 切换）
- 行号显示（`-l` 或 `L` 切换）
- 翻页页码
- `u`/`d` 半页滚动，`b`/`f` 翻页
- 文件修改时间显示（中文月份格式）
- HTTP 服务器子命令占位（`mdr serve`）

#### 技术栈
- Rust 1.94.0
- comrak 0.33（Markdown 解析）
- ratatui 0.29 + crossterm 0.28（TUI）
- syntect 5.3（语法高亮）
- clap 4.6（CLI）
- walkdir 2（文件扫描）

---

## 2. 进度

### ✅ 已完成

| 模块 | 功能 | 优先级 |
|------|------|--------|
| Markdown 渲染 | 标题 / 段落 / 代码块 / 列表 / 引用 / 分割线 / 粗斜体 / 行内代码 / 链接 / 图片 / 表格 / 任务列表 | P0 |
| 语法高亮 | syntect 支持多种语言，base16-ocean.dark 主题 | P0 |
| 内置 Pager | ↑↓/jk 滚动, PgUp/PgDn 翻页, g/G 首尾, 状态栏 | P0 |
| 文件选择器 | 扫描 .md, 卡片列表, 文件计数, 翻页页码 | P0 |
| 搜索 | `/` 搜索, 大小写不敏感, 匹配计数, 红色高亮 | P0 |
| 帮助抽屉 | `?` 展开/收起快捷键一览 | P1 |
| 鼠标支持 | 滚轮滚动, 点击/拖拽滚动条 | P1 |
| 行号 | `-l` 参数或 `L` 键切换 | P1 |
| 半页滚动 | `u`/`d` 半页, `b`/`f` 全页 | P1 |
| 文件选择器循环 | 选文件 → 阅读 → q → 返回选择器 | P0 |
| 左右方向键 | `←` 返回, `→` 打开 | P1 |
| MDR 标题栏 | 蓝底白字 logo, 文档计数, 距离左侧 3 列, 顶部空 1 行 | P2 |
| 卡片样式 | 选中蓝色竖线 + 蓝色文字, 未选中仅缩进 | P2 |
| 底部菜单栏 | 无背景, ` · ` 分隔, 蓝色 MDR logo | P2 |
| 日期格式 | 中文月份 "25 7月 2026  17:05" | P2 |
| CI/CD | 18 个单元测试 | P1 |
| HTTP 服务器 | `mdr serve --port 8080`（axum + minijinja） | P1 |
| VitePress CSS 变量 | 完整 `--vp-c-*` 体系，暗色/亮色 | P1 |
| 页面导航 | On This Page：小屏下拉/大屏右侧边栏 | P1 |
| 响应式布局 | 三档断点：768/1280/1440 | P2 |
| 开发模式 | `MDR_DEV=1` 从文件系统加载模板 | P2 |
| 代码块标题栏 | 双层布局：语言标题 + 代码区 + 复制按钮 | P1 |
| 代码组 | 标签页切换 `::: code-group` | P1 |
| 自定义容器 | info/tip/warning/danger/details 等 | P1 |
| 字体对齐 VitePress | h1:28-32px, 正文:16px, code:0.875em | P2 |
| SSE 热重载 | MDR_DEV=1 文件监听 + 浏览器自动刷新 | P2 |

### ❌ 未完成

| 功能 | 说明 | 优先级 |
|------|------|--------|
| 双主题代码高亮 | Shiki 风格 --shiki-light/--shiki-dark | P2 |
| 配置文件 | `~/.config/mdr/config.toml` | P3 |
| 双主题代码高亮 | Shiki 风格 --shiki-light/--shiki-dark | P2 |
| 脚注 / 定义列表 | 额外 Markdown 语法 | P3 |
| 行跳转 | `:` 进入行号跳转 | P2 |
| 正则搜索 | 搜索支持正则表达式 | P3 |
| 图片渲染 | 终端内渲染图片（需 kitty/protocol 支持） | P3 |
| 导出 PDF | 将 Markdown 导出为 PDF | P3 |
| 发布 | crates.io + Homebrew | P3 |

### 🔮 待定

| 功能 | 说明 |
|------|------|
| 暗色/亮色主题切换 | 配置文件 + 命令行参数 |
| 多语言界面 | i18n 支持 |
| 文件监视 | `notify` 监听文件变更自动重载 |
| WebSocket 热更新 | HTTP 模式下文件变更自动推送 |
| brew 发布 | Homebrew formula |
| crates.io 发布 | `cargo publish` |

---

## 3. 讨论记录

### 2026-07-26 — 项目启动 & Phase 1

**目标**: 用 Rust 写一个类似 glow 的终端 Markdown 渲染工具，不依赖 less/more

**决策**:
- 使用 comrak 解析 Markdown AST（而非 termimad，保留更多控制权）
- 使用 ratatui 构建 TUI（自带 pager 能力）
- 使用 syntect 做语法高亮
- 项目名 `mdr`（Markdown Renderer）

**架构**:
```
mdr/
├── src/
│   ├── main.rs           CLI 入口
│   ├── cli.rs            参数定义
│   ├── render/terminal.rs  Markdown → 终端样式
│   ├── tui/app.rs         TUI pager
│   └── lib.rs             库入口
```

### 2026-07-26 — Phase 2: 搜索 + 行号 + 文件选择器

**决策**:
- 搜索风格：编辑器风格（Enter 循环切换候选，Esc 退出）
- 匹配高亮使用红色（后改为蓝色，与 logo 统一）
- 文件选择器：扫描 .md，列表选择

**新增功能**:
- 实时搜索过滤
- 匹配计数与当前匹配指示
- 翻页页码
- 方向键导航（← 返回, → 打开）

### 2026-07-26 — UI 打磨

**决策过程**:
1. 选中的卡片显示 `▸` → 改为 blockquote 风格的 `│` 竖线
2. 表格布局 → 卡片式布局（3行：竖线+文件名 / 竖线+时间 / 空行）
3. 所有卡片都有竖线 → 仅选中项有竖线
4. 顶部 MDR 标题蓝底白字 + 文档计数
5. 搜索栏集成到顶部（MDR → FIND:）→ 改为底部菜单栏统一显示
6. 日期格式从 YYYY-MM-DD → 25 7月 2026

**底部菜单栏演化**:
1. 有背景色 + 边框 → 无背景色 + ` · ` 分隔
2. 快捷键从顶部移到底部
3. 帮助抽屉从菜单栏下方弹出（菜单栏在上，帮助在下）
4. 帮助抽屉表格对齐（固定列宽）

### 2026-07-26 — 帮助抽屉

**参考 glow 的帮助面板**:
```
k/↑    up          g/home  go to top
j/↓    down        G/end   go to bottom
...
```

**注意**: 帮助菜单中的快捷键必须与实际功能一致。之前列了 `u`/`d` 半页滚动但未实现，需要补上功能再显示。

### 2026-07-26 — vs glow 差异

| 维度 | glow | mdr |
|------|------|-----|
| 语言 | Go | Rust |
| 分页 | 依赖 less/more | 内置 ratatui pager |
| 鼠标 | ❌ | ✅ 滚轮/点击/拖拽 |
| 文件选择器 | 简单列表 | 卡片式 + 翻页 + 过滤 |

---

## 4. 认知修正（踩坑记录）

> Agent 在开发过程中遇到的问题和解决方案，作为后续开发的认知参考。

### 4.1 comrak API — AstNode 生命周期

**问题**: comrak 0.33 的 `AstNode` 只接受 1 个生命周期参数，而不是 2 个。

```
// ❌ 错误
fn render_node(node: &AstNode<'a, 'a>)

// ✅ 正确
fn render_node(node: &AstNode<'a>)
```

**教训**: 编译错误时先检查上游库的 API 版本。`AstNode` 在不同版本的 comrak 中签名可能不同。

### 4.2 ratatui — backend 路径

**问题**: ratatui 0.29 中 `CrosstermBackend` 的路径是 `ratatui::backend::CrosstermBackend`，不是 `ratatui::backends::CrosstermBackend`。

```
// ❌ 错误
use ratatui::backends::CrosstermBackend;

// ✅ 正确
use ratatui::backend::CrosstermBackend;
```

### 4.3 ratatui — Terminal::into_inner() 不存在

**问题**: `Terminal` 在 ratatui 0.29 中没有 `into_inner()` 方法。

```
// ❌ 错误
let stdout = terminal.into_inner();

// ✅ 正确
terminal.backend_mut().execute(...)?;
```

### 4.4 ratatui — `Layout::areas()` 返回数组

**问题**: `Layout::vertical(...).areas(area)` 返回 `[Rect; N]`（数组），不是元组。

```
// ❌ 错误
let (a, b, c) = layout.areas(area);

// ✅ 正确（Rust 1.94 支持）
let [a, b, c] = layout.areas(area);
```

### 4.5 搜索栏消失 Bug

**问题**: 按 `/` 进入搜索模式后搜索栏不显示。

**原因**: 搜索栏使用了 `Block::default().borders(Borders::TOP)`，Block 占据 2 行空间，
但状态栏只分配了 `Constraint::Length(1)`（1 行）。搜索栏被裁掉了。

**修复**: 移除搜索栏的边框 Block，仅用 Paragraph 本身。同时修正光标位置 `y` 坐标：
之前 `area.y + 1` 是因为边框占了一行，现在改为 `area.y`。

**教训**: 当布局使用 `Constraint::Length(N)` 时，确保内部渲染内容不超过 N 行。
Block 的边框会额外增加行数。

### 4.6 ratatui Text vs Line — 多行文本

**问题**: 帮助抽屉的快捷键列表显示为一整行，没有正确换行。

**原因**: 在 spans 中嵌入 `\n` 字符不会在 `Line` 中产生换行。`Line` 是单行的，
`Text` 才是由多个 `Line` 组成的多行文本。

```
// ❌ 错误 — \n 不生效
let spans = vec![...];  // 包含 \n
let text = Text::from(Line::from(spans));

// ✅ 正确 — 每个 Line 是一行
let rows: Vec<Line> = vec![line1, line2, ...];
let text = Text::from(rows);
```

**教训**: ratatui 中 `Line` = 一行，`Text` = 多行（由多个 `Line` 组成）。
需要在 span 中换行 → 用多个 `Line`。

### 4.7 卡片布局的定位问题

**问题**: `Rect::new` 的 `width` 和 `height` 参数是 `u16` 类型，但计算时用了 `usize`。

```
// ❌ 错误 — 类型不匹配
Rect::new(x, y, card_w + 2, 1)  // card_w 是 usize

// ✅ 正确 — 显式转换
Rect::new(x, y, (card_w + 2) as u16, 1)
```

### 4.8 HTML 行内元素渲染

**问题**: 在 `collect_inline` 中处理 `NodeValue::Link` 时，尝试直接修改 `span.style`，
但 `span.style` 同时被借用导致编译错误。

```
// ❌ 错误
let _ = std::mem::replace(&mut span.style, span.style.fg(Color::Blue)...);

// ✅ 正确 — 先读取，后写入
let new_style = span.style.fg(Color::Blue)...;
span.style = new_style;
```

### 4.9 嵌套列表缩进

**问题**: 嵌套列表的子项没有正确缩进。

**原因**: 列表缩进通过 `list_indent` 栈管理。`render_list_item` 和 `render_node`
（List handler）各自管理缩进，叠加后导致双重缩进。

**修复**: `render_list_item` 遇到嵌套 List 时不再自行管理 `list_indent`，
而是完全委托给 `render_node` 的 List handler。

### 4.10 帮助抽屉方向

**问题**: 最初实现时帮助内容在菜单栏上方，但 glow 的交互是菜单栏在帮助上方。

**修复**: 调整 layout 顺序为 `[content, status, help]`，渲染顺序也相应调整。

**教训**: 模仿 UI 组件时，先确认清楚组件的空间位置关系（上下顺序），再编码。

### 4.11 搜索栏与底部菜单栏的布局冲突

**问题**: 改为抽屉式菜单栏后，搜索栏和状态栏共用同一个 `status_area`（1 行高度）。
搜索栏渲染在状态栏位置，两者视觉不同。

**当前方案**: 普通模式 → 渲染状态栏；搜索模式 → 渲染搜索栏（覆盖状态栏）。

**潜在改进**: 搜索模式下可以保留路径信息，将 `/query` 集成到菜单栏右侧。

### 4.12 文件路径在状态栏的传递

**问题**: pager 最初不感知文件路径，无法在底部菜单栏显示文件名。

**修复**: 在 `PagerConfig` 中添加 `file_path: String` 字段，`main.rs` 中传入。

**教训**: 需要跨模块传递数据时，优先考虑 Config 结构体，避免修改函数签名链。

### 4.13 中文字符宽度 — byte vs char vs display

**问题**: 表格列宽用 `str::len()` 计算（字节长度），中文 1 字 = 3 字节，对齐全乱。

**修复**: 使用 `unicode-width` crate 的 `UnicodeWidthStr::width()` 计算终端显示宽度。

**教训**: 涉及文本排版时永远用 `width()` 而非 `len()`。

### 4.14 渲染器和 TUI 内容区宽度不一致

**问题**: 渲染器用 `viewport - 1` 换行，TUI 内容区 `viewport - 2`（左右 padding），
差 1 列导致右边缘与滚动条重叠。

**修复**: 统一减 2，渲染器存储 `content_width = viewport - 2`。

**教训**: 涉及多模块的尺寸常量必须统一管理，避免各算各的。

### 4.15 comrak 表格扩展默认关闭

**问题**: `ComrakOptions::default()` 不启用 GFM 表格解析，表格被当成普通文本。

**修复**: `options.extension.table = true;`

**教训**: 使用 comrak 时检查 `extension` 和 `render` 字段是否开启所需特性。

### 4.16 搜索栏消失 — Block 边框占空间

**问题**: 搜索栏使用 `.block(Block::default().borders(Borders::TOP))`，Block 在
内容外额外占用 1 行，但状态栏只分配 `Length(1)`，搜索栏被裁切。

**修复**: 移除 Block 边框，搜索栏只保留 Paragraph。

**教训**: `Constraint::Length(N)` 布局下，内部元素总高度不能超过 N 行。

### 4.17 自定义容器类型解析 — trim_start_matches 残留空格

**问题**: `trim_start_matches(':')` 移除 `:::` 后留下空格，导致 `splitn` 把空格
当分隔符，类型名变空字符串。

**修复**: 加 `.trim()` 再 split。

**教训**: 字符串清理要完整，不能只 trim 部分字符。

---

## 5. 编码规范

### 5.1 注释语言

全部注释优先使用**中文**。对于关键名词、API 名称、技术术语等，保留原始英文。

```rust
// ✅ 正确
/// 计算列的最大显示宽度（使用 unicode-width 处理 CJK 字符）
fn calc_col_widths(spans: &[Span]) -> Vec<usize> { ... }

// ❌ 错误 — 关键名词被翻译
fn ji_suan_lie_kuan_du(...)  // 不要拼音或中译名

// ❌ 错误 — 纯英文注释（在中文为主的代码中增加认知负担）
// Calculate the maximum column width
```

### 5.2 开发范式

采用 **Package + OOP（面向对象）** 的开发思维：

- **Package（模块化）**：按功能划分模块（`render/`、`tui/`），模块间通过
  `pub` 接口通信，内部实现隐藏。
- **OOP（面向对象）**：用 `struct` + `impl` 封装状态和行为，通过组合而非
  继承复用代码。数据和方法绑定在一起。

```rust
// ✅ 正确 — struct 封装状态，impl 封装行为
pub struct PagerConfig {
    pub show_line_numbers: bool,
    pub from_picker: bool,
    pub file_path: String,
}

impl App {
    pub fn run(&mut self) -> PagerExit { ... }
    fn handle_key(&mut self, key: KeyEvent) { ... }
    fn render(&self, frame: &mut Frame) { ... }
}
```

### 5.3 文档及时性

每次开发完成后，**立即更新**以下文档：

1. `CHANGELOG` — AGENTS.md 第一节，记录新增 / 修复 / 变更
2. `进度` — AGENTS.md 第二节，更新已完成 / 未完成清单
3. `认知修正` — AGENTS.md 第四节，记录踩坑和解决方案
4. `README.md` — 项目主页，保持功能列表和使用说明与代码同步

### 5.4 文档排版

编辑 Markdown 文档时：

- **段落间保留 1 个空行**，不要多个空行堆叠
- **列表项之间不要空行**（除非语义上需要分组）
- **代码块前后保留 1 个空行**，与上下文隔开
- 移除无意义的空行，保证阅读流畅性

```markdown
// ✅ 正确 — 简洁清晰
## 标题

段落内容。

- 列表项 1
- 列表项 2

```

// ❌ 错误 — 多余空行
## 标题


段落内容。


- 列表项 1


- 列表项 2
```

---

> 上次更新: 2026-07-26
> 下一版本计划: v0.3.0 — 双主题代码高亮 + 配置文件
