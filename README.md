# MDR — Markdown 终端查看器

> 一个现代化的、在终端阅读markdown的阅读器 和 markdown 文件服务器。
> 不需要 `less`，不需要 `more`，不依赖任何外部命令，不需要任何其他第三方服务器，即可实现markdown的在线部署和阅读。

![Rust](https://img.shields.io/badge/Rust-1.94%2B-orange)
![license](https://img.shields.io/badge/license-MIT-blue)

---

## 功能

### 终端阅读器

- **Markdown 渲染** — 标题、段落、代码块（语法高亮）、列表、表格、任务列表（☐/☑）、引用块、分割线、粗体/斜体/删除线、行内代码、链接、图片
- **YAML Frontmatter** — 文档头部元数据识别，灰色 dim 样式显示
- **内置 Pager** — `↑↓`/`jk` 滚动，`PgUp`/`PgDn`/`b`/`f` 翻页，`u`/`d` 半页
- **搜索** — `/` 搜索，`n`/`N` 跳转匹配
- **帮助抽屉** — `?` 展开/关闭快捷键面板
- **行号** — `-l` 或 `L` 切换
- **鼠标支持** — 滚轮滚动、点击拖拽滚动条、`Ctrl+m` 切换鼠标捕获
- **Ctrl+点击** — 在浏览器中打开链接

### Markdown 在线服务器

- markdown 文件索引和查看
- 支持常用的 `markdown` 语法，和 __代码组__ （`::: code-group`） 、自定义容器：info / tip / warning / danger / details
- **YAML Frontmatter** — 自动解析 `categories`/`tags`/`date`/`updated`，底部状态栏显示分类和更新时间
- **页面导航** — 右侧 ToC 侧边栏（h2+），窄屏可折叠下拉菜单


## 安装

### 从 Release 下载支持你平台的二进制包即可
- 支持 Mac \ Windows \ Linux (x64/arm) 平台架构

### 从源码编译
```bash
git clone https://github.com/titor/mdr.git
cd mdr
cargo build --release
cp target/release/mdr /usr/local/bin/
```

#### 系统要求
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

# 开发模式（文件监听 + 浏览器自动刷新）
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

## 与 最流行的 markdown终端查看器 glow 对比

| 特性 | glow | mdr |
|------|------|-----|
| 语言 | Go | Rust |
| 外部 pager | 依赖 `less`/`more` | ✅ 内置 pager |
| 鼠标支持 | ❌ | ✅  |
| 表格渲染 | 纯文本 | ✅  |
| 在线预览 | ❌ | ✅ `mdr serve` |
| 代码组 | ❌ | ✅ 标签页切换 |
| 自定义容器 | ❌ | ✅ info/tip/warning/danger |
| 任务列表 | ❌ | ✅ ☐/☑ |
| 二进制体积 | ~15 MB | ~5 MB (release) |

---


## 开源协议
MIT
