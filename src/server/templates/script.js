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

// 页面加载时恢复主题
document.addEventListener('DOMContentLoaded', () => {
  const saved = localStorage.getItem('mdr-theme');
  if (saved) setTheme(saved);
});
