# mdr — Markdown Renderer

> 一个现代化的、自包含的终端 Markdown 渲染器，内置 pager。
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
- **鼠标支持** — 滚轮滚动、点击拖拽滚动条、`Ctrl+m` 切换鼠标捕获用于文字选择
- **Ctrl+点击** — 按住 `Ctrl` 点击链接，在浏览器中打开

### 文件选择器（首页）

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

### 表格

```
┌──────────┬──────────────────────────────────┬──────────┐
│ Name     │ Description                      │ Status   │
├──────────┼──────────────────────────────────┼──────────┤
│ Alice    │ A very long description that     │ Active   │
│          │ wraps across multiple lines      │          │
├──────────┼──────────────────────────────────┼──────────┤
│ Bob      │ Short                            │ Inactive │
└──────────┴──────────────────────────────────┴──────────┘
```

- 完整四周边框（`┌─┬─┐│├─┼─┤└─┴─┘`）
- 列宽自动分配，填满视窗
- 自动换行 + 垂直居中
- 多字节字符支持（中日韩）

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

# 打开文件选择器（浏览当前目录）
mdr

# 显示行号
mdr -l README.md

# HTTP 服务器

```bash
mdr serve --port 8080
mdr serve --port 8080 --dir ./docs
```
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
| 二进制体积 | ~15 MB | ~5 MB (release) |

---

## 项目状态

**当前版本**: v0.1.0

- ✅ Markdown 渲染，含语法高亮
- ✅ 内置 TUI pager
- ✅ 文件选择器，含搜索和翻页
- ✅ 表格渲染，含边框和自动换行
- ✅ 段落自动换行
- ✅ 任务列表（☐/☑）
- ✅ 鼠标交互
- ✅ 搜索匹配高亮
- ✅ HTTP 服务器（`mdr serve`）
- 📝 配置文件支持 — 计划中

---

## 开源协议

MIT
