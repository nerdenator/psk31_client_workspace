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
import { setupAudioPanel } from './components/audio-panel';
import { setupMenuEvents } from './services/event-handlers';
import { startFftBridge, listenAudioStatus } from './services/audio-bridge';

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

  // Wire up audio bridge: FFT events → waterfall display
  if (waterfall) {
    startFftBridge(waterfall);
    listenAudioStatus(waterfall);
  }
});
