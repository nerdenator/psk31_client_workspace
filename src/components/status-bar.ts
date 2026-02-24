/** Status bar — drives connection indicators.
 *
 * Subscribes to app-state for serial/audio connection changes.
 * Calls hydrateFromBackend() on init so state is correct after a reload.
 */

import { onSerialChanged, onAudioChanged, hydrateFromBackend } from '../services/app-state';

export async function setupStatusBar(): Promise<void> {
  const serialDot = document.querySelector('#statusbar-serial .status-dot') as HTMLElement | null;
  const serialText = document.querySelector(
    '#statusbar-serial .status-text',
  ) as HTMLElement | null;
  const audioDot = document.querySelector('#statusbar-audio .status-dot') as HTMLElement | null;
  const audioText = document.querySelector('#statusbar-audio .status-text') as HTMLElement | null;

  function updateSerialIndicator(connected: boolean, portName: string | null): void {
    if (serialDot) {
      serialDot.classList.toggle('connected', connected);
      serialDot.classList.toggle('disconnected', !connected);
    }
    if (serialText) {
      serialText.classList.toggle('connected', connected);
      serialText.classList.toggle('disconnected', !connected);
      serialText.textContent = connected && portName ? truncate(portName, 18) : 'CAT';
    }
  }

  function updateAudioIndicator(streaming: boolean, deviceName: string | null): void {
    if (audioDot) {
      audioDot.classList.toggle('connected', streaming);
      audioDot.classList.toggle('disconnected', !streaming);
    }
    if (audioText) {
      audioText.classList.toggle('connected', streaming);
      audioText.classList.toggle('disconnected', !streaming);
      audioText.textContent = streaming && deviceName ? truncate(deviceName, 14) : 'Audio';
    }
  }

  // Subscribe to connection state changes
  onSerialChanged(updateSerialIndicator);
  onAudioChanged(updateAudioIndicator);

  // Seed state from Rust — makes status bar correct after a webview reload
  await hydrateFromBackend();
}

function truncate(s: string, maxLen: number): string {
  return s.length <= maxLen ? s : s.slice(0, maxLen - 1) + '\u2026';
}
