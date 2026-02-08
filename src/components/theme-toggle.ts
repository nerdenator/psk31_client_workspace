/** Theme switching with localStorage persistence */

export function setTheme(theme: 'light' | 'dark'): void {
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.setItem('psk31-theme', theme);

  const icon = document.querySelector('.theme-toggle-icon') as HTMLElement;
  if (icon) {
    icon.textContent = theme === 'dark' ? '☀' : '☽';
  }
}

export function getCurrentTheme(): 'light' | 'dark' {
  return (document.documentElement.getAttribute('data-theme') as 'light' | 'dark') || 'light';
}

export function setupThemeToggle(): void {
  const toggle = document.getElementById('theme-toggle') as HTMLButtonElement;

  // Priority: saved preference > system preference > light (default)
  const savedTheme = localStorage.getItem('psk31-theme') as 'light' | 'dark' | null;
  const systemPreference = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' :
                           window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : null;
  const initialTheme = savedTheme || systemPreference || 'light';

  setTheme(initialTheme as 'light' | 'dark');

  toggle?.addEventListener('click', () => {
    const next = getCurrentTheme() === 'dark' ? 'light' : 'dark';
    setTheme(next);
  });
}
