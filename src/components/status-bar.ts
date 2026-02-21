/** Status bar — drives connection indicators and signal level meter.
 *
 * Subscribes to app-state for serial/audio connection changes.
 * Listens to the 'signal-level' Tauri event for the 5-bar signal meter.
 * Calls hydrateFromBackend() on init so state is correct after a reload.
 */

import { listen } from '@tauri-apps/api/event';
import { onSerialChanged, onAudioChanged, hydrateFromBackend } from '../services/app-state';

interface SignalLevelPayload {
  level: number;
}

export async function setupStatusBar(): Promise<void> {
  const signalBars = Array.from(
    document.querySelectorAll('.signal-bars .signal-bar'),
  ) as HTMLElement[];

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

  function updateSignalBars(level: number): void {
    const activeBars = Math.round(level * signalBars.length);
    signalBars.forEach((bar, i) => {
      bar.classList.toggle('active', i < activeBars);
    });
  }

  // Subscribe to connection state changes
  onSerialChanged(updateSerialIndicator);
  onAudioChanged((streaming, deviceName) => {
    updateAudioIndicator(streaming, deviceName);
    // Clear signal bars when audio stops
    if (!streaming) updateSignalBars(0);
  });

  // Seed state from Rust — makes status bar correct after a webview reload
  await hydrateFromBackend();

  // Listen for signal level events from the audio thread
  const unlisten = await listen<SignalLevelPayload>('signal-level', (event) => {
    updateSignalBars(event.payload.level);
  });

  window.addEventListener('beforeunload', () => void unlisten());
}

function truncate(s: string, maxLen: number): string {
  return s.length <= maxLen ? s : s.slice(0, maxLen - 1) + '\u2026';
}
