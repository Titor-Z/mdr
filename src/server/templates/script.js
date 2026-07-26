// 主题切换（VitePress 风格，使用 .dark class）
function getTheme() {
  return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
}

function setTheme(theme) {
  const root = document.documentElement;
  if (theme === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
  localStorage.setItem('mdr-theme', theme);
  const btn = document.getElementById('theme-toggle');
  if (btn) btn.textContent = theme === 'dark' ? '🌙 暗色' : '☀️ 亮色';
}

function toggleTheme() {
  const next = getTheme() === 'dark' ? 'light' : 'dark';
  setTheme(next);
}

// 复制代码按钮
document.addEventListener('click', function(e) {
  const btn = e.target.closest('.copy');
  if (!btn) return;
  
  // 找到最近的 pre > code 或 pre
  const container = btn.closest('.language-') || btn.closest('.vp-code-block-title');
  if (!container) return;
  const pre = container.querySelector('pre');
  if (!pre) return;
  
  // 提取代码文本（去掉高亮 span 标签）
  const code = pre.textContent || pre.innerText || '';
  
  // 复制到剪贴板
  navigator.clipboard.writeText(code).then(() => {
    btn.setAttribute('data-copied', 'true');
    btn.title = '已复制';
    setTimeout(() => {
      btn.removeAttribute('data-copied');
      btn.title = '复制代码';
    }, 2000);
  }).catch(() => {
    // fallback: 使用 textarea
    const ta = document.createElement('textarea');
    ta.value = code;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
    btn.setAttribute('data-copied', 'true');
    btn.title = '已复制';
    setTimeout(() => {
      btn.removeAttribute('data-copied');
      btn.title = '复制代码';
    }, 2000);
  });
});

// 代码组标签页切换
document.addEventListener('click', function(e) {
  const tab = e.target.closest('.vp-code-group .tab');
  if (!tab) return;
  const group = tab.closest('.vp-code-group');
  if (!group) return;
  
  // 取消所有 tab 的激活
  group.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
  group.querySelectorAll('.code-block').forEach(b => b.classList.remove('active'));
  
  // 激活当前 tab
  tab.classList.add('active');
  const idx = Array.from(group.querySelectorAll('.tab')).indexOf(tab);
  const blocks = group.querySelectorAll('.code-block');
  if (blocks[idx]) blocks[idx].classList.add('active');
});

// 页面导航下拉菜单
function togglePageNav() {
  const dd = document.getElementById('page-nav-dropdown');
  const ch = document.getElementById('page-nav-chevron');
  if (!dd || !ch) return;
  const isOpen = dd.classList.toggle('open');
  ch.classList.toggle('open', isOpen);
}

function closePageNav() {
  const dd = document.getElementById('page-nav-dropdown');
  const ch = document.getElementById('page-nav-chevron');
  if (!dd || !ch) return;
  dd.classList.remove('open');
  ch.classList.remove('open');
}

// 点击外部关闭
function closePageNavOnOutside(event) {
  const nav = document.querySelector('.page-nav');
  if (!nav) return;
  if (!nav.contains(event.target)) {
    closePageNav();
  }
}

document.addEventListener('click', closePageNavOnOutside);

// 页面加载时恢复主题
document.addEventListener('DOMContentLoaded', () => {
  const saved = localStorage.getItem('mdr-theme');
  if (saved) setTheme(saved);
});
