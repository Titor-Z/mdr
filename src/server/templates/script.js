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

// 窄屏 ToC 弹出层
function toggleTocMobile() {
  const overlay = document.getElementById('toc-mobile');
  const chevron = document.getElementById('toc-chevron');
  if (!overlay || !chevron) return;
  const isOpen = overlay.classList.toggle('open');
  chevron.classList.toggle('open', isOpen);
  document.body.style.overflow = isOpen ? 'hidden' : '';
}

function closeTocMobile() {
  const overlay = document.getElementById('toc-mobile');
  const chevron = document.getElementById('toc-chevron');
  if (!overlay || !chevron) return;
  overlay.classList.remove('open');
  chevron.classList.remove('open');
  document.body.style.overflow = '';
}

function closeTocMobileOutside(event) {
  if (event.target === event.currentTarget) {
    closeTocMobile();
  }
}

// 按下 Esc 关闭 ToC
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') closeTocMobile();
});

// 页面加载时恢复主题
document.addEventListener('DOMContentLoaded', () => {
  const saved = localStorage.getItem('mdr-theme');
  if (saved) setTheme(saved);
});
