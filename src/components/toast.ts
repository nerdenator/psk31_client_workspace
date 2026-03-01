/** Toast notifications — brief auto-dismissing messages for hardware errors */

export type ToastType = 'error' | 'warning' | 'info';

const DEFAULT_DURATION_MS = 3500;

export function showToast(
  message: string,
  type: ToastType = 'error',
  durationMs = DEFAULT_DURATION_MS,
): void {
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.textContent = message;
  document.body.appendChild(toast);

  // Trigger enter animation on next frame
  requestAnimationFrame(() => toast.classList.add('toast-visible'));

  setTimeout(() => {
    toast.classList.remove('toast-visible');
    toast.addEventListener('transitionend', () => toast.remove(), { once: true });
  }, durationMs);
}
