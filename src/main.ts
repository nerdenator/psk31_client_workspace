/**
 * PSK-31 Client - Main Application Entry
 *
 * Phases 1-3: Serial/CAT, audio input, and waterfall display.
 */

import { WaterfallDisplay } from './components/waterfall';
import { setupRxDisplay } from './components/rx-display';
import { setupTxInput } from './components/tx-input';
import { setupTxButtons } from './components/control-panel';
import { setupWaterfallClick } from './components/waterfall-controls';
import { setupThemeToggle } from './components/theme-toggle';
import { setupSerialPanel } from './components/serial-panel';
import { setupAudioPanel, resetAudioPanel } from './components/audio-panel';
import { setupStatusBar } from './components/status-bar';
import { showToast } from './components/toast';
import { setupMenuEvents } from './services/event-handlers';
import { startFftBridge, listenAudioStatus } from './services/audio-bridge';
import { startRxBridge } from './services/rx-bridge';
import { startSerialBridge } from './services/serial-bridge';
import { appendRxText } from './components/rx-display';

window.addEventListener('DOMContentLoaded', () => {
  const canvas = document.getElementById('waterfall-canvas') as HTMLCanvasElement;
  let waterfall: WaterfallDisplay | null = null;

  if (canvas) {
    waterfall = new WaterfallDisplay(canvas);
    waterfall.start();
  }

  setupTxInput();
  setupTxButtons();
  setupRxDisplay();
  setupWaterfallClick();
  setupThemeToggle();
  setupSerialPanel();
  setupAudioPanel();
  setupMenuEvents();

  // Status bar — after serial/audio panels so they can fire setters on connect
  setupStatusBar().catch((err) => {
    console.error('Failed to set up status bar:', err);
  });

  // Wire up audio bridge: FFT events → waterfall; error status → toast + reset UI
  if (waterfall) {
    startFftBridge(waterfall);
    listenAudioStatus(waterfall, (status) => {
      if (status.startsWith('error:')) {
        resetAudioPanel();
        showToast('Audio device lost', 'error');
      }
    });
  }

  // Wire up serial bridge: backend-initiated disconnect → toast + reset UI
  startSerialBridge().catch((err) => {
    console.error('Failed to start serial bridge:', err);
  });

  // Wire up RX bridge: decoded text events → RX display
  startRxBridge(appendRxText).catch((err) => {
    console.error('Failed to start RX bridge:', err);
  });
});
