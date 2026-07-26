// 主题切换
function getTheme() {
  return document.documentElement.getAttribute('data-theme') || 'dark';
}

function setTheme(theme) {
  document.documentElement.setAttribute('data-theme', theme);
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
