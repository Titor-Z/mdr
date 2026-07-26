# mdr — Markdown Renderer

> 一个现代化的、自包含的终端 Markdown 渲染器，内置 pager + HTTP 服务器。
> 不需要 `less`，不需要 `more`，不依赖任何外部命令。

![Rust](https://img.shields.io/badge/Rust-1.94%2B-orange)
![license](https://img.shields.io/badge/license-MIT-blue)

---

## 功能

### 终端阅读器

- **Markdown 渲染** — 标题、段落、代码块（语法高亮）、列表、表格、任务列表（☐/☑）、引用块、分割线、粗体/斜体/删除线、行内代码、链接、图片
- **内置 Pager** — `↑↓`/`jk` 滚动，`PgUp`/`PgDn`/`b`/`f` 翻页，`u`/`d` 半页
- **搜索** — `/` 搜索，`n`/`N` 跳转匹配
- **帮助抽屉** — `?` 展开/关闭快捷键面板
- **行号** — `-l` 或 `L` 切换
- **鼠标支持** — 滚轮滚动、点击拖拽滚动条、`Ctrl+m` 切换鼠标捕获
- **Ctrl+点击** — 在浏览器中打开链接

### HTTP 服务器

```
┌───────────────────────────────────────────────┐
│                     MDR                       │
├───────────────────────────────────────────────┤
│  │ README.md                                  │  ← 首页卡片列表
│  │ 25 7月 2026  14:30                         │
│                                               │
│  │ docs/guide.md                              │
│  │ 17 3月 2026  18:52                         │
│                                               │
│  共 5 个文档                                  │
├───────────────────────────────────────────────┤
│  © 2026 mdr                         🌙 暗色  │
└───────────────────────────────────────────────┘
```

- **首页** — 卡片式文件列表，竖线 + 文件名 + 日期
- **详情页** — 左侧文件树 + 中间文章 + 右侧页面导航
- **主题** — 暗色/亮色切换，CSS 变量，localStorage 持久化
- **页面导航** — 小屏「页面导航 ▾」下拉菜单，大屏右侧固定目录
- **响应式** — 三档断点适配手机/平板/桌面

### 代码块

```
┌─────────────────────────────────────┐
│  rust                           [📋]│  ← 标题栏 + 复制按钮
├─────────────────────────────────────┤
│  fn greet(name: &str) -> String {   │
│      format!("Hello!", name)        │
│  }                                   │
└─────────────────────────────────────┘
```

- VitePress 风格双层布局：标题栏 + 代码区
- syntect 语法高亮（Solarized dark 主题）
- 语言标签 + 悬浮复制按钮
- 代码组标签页切换（`::: code-group`）
- 自定义容器：info / tip / warning / danger / details

### 文件选择器（终端）

```
  █ MDR █

  10 documents · page 1

  │ README.md
  │ 25 7月 2026  14:30

    notes/git命令集合.md
    17 3月 2026  18:52

  FIND: query  ·  → open  ·  ← back  ·  q quit
```

- 实时搜索过滤
- 卡片式布局，显示修改时间
- 翻页（`PgUp`/`PgDn`，`gg`/`GG`）
- 中文日期格式

---

## 安装

### 从源码编译

```bash
git clone https://github.com/titor/mdr.git
cd mdr
cargo build --release
cp target/release/mdr /usr/local/bin/
```

### 系统要求

- Rust 1.73+
- 支持真彩色（true color）的终端（用于语法高亮）

---

## 使用

```bash
# 直接打开文件
mdr README.md

# 打开文件选择器
mdr

# 显示行号
mdr -l README.md

# HTTP 服务器
mdr serve --port 8080
mdr serve --port 8080 --dir ./docs

# 开发模式（模板从文件系统加载，改完重启即可）
MDR_DEV=1 mdr serve --port 8080
```

### 快捷键

| 按键 | 功能 |
|------|------|
| `↑`/`k` | 上滚一行 |
| `↓`/`j` | 下滚一行 |
| `PgUp`/`b` | 上一页 |
| `PgDn`/`f` | 下一页 |
| `u`/`d` | 半页上/下 |
| `g`/`G` | 顶部/底部 |
| `/` | 搜索 |
| `n`/`N` | 下一个/上一个匹配 |
| `?` | 切换帮助抽屉 |
| `L` | 切换行号 |
| `Ctrl+m` | 切换鼠标捕获 |
| `Ctrl+点击` | 在浏览器中打开链接 |
| `q`/`Esc` | 退出 / 返回 |

---

## 与 glow 对比

| 特性 | glow | mdr |
|------|------|-----|
| 语言 | Go | Rust |
| 外部 pager | 依赖 `less`/`more` | 内置 ratatui pager |
| 鼠标支持 | ❌ | ✅ 滚轮、点击、拖拽 |
| 表格渲染 | 纯文本 | ✅ 四周边框 + 自动换行 |
| 文件选择器 | 简单列表 | 卡片布局 + 翻页 |
| HTTP 服务器 | ❌ | ✅ `mdr serve` |
| 代码组 | ❌ | ✅ 标签页切换 |
| 自定义容器 | ❌ | ✅ info/tip/warning/danger |
| 任务列表 | ❌ | ✅ ☐/☑ |
| 二进制体积 | ~15 MB | ~5 MB (release) |

---

## 项目状态

**当前版本**: v0.2.0

- ✅ Markdown 渲染，含语法高亮
- ✅ 内置 TUI pager
- ✅ 文件选择器，含搜索和翻页
- ✅ HTTP 服务器，VitePress 风格页面
- ✅ 代码块标题栏 + 复制按钮
- ✅ 代码组（标签页切换）
- ✅ 自定义容器（info/tip/warning/danger/details）
- ✅ 页面导航（On This Page）
- ✅ 响应式布局
- 🚧 SSE 热重载 — 开发中
- 📝 配置文件支持 — 计划中

---

## 开源协议

MIT
