/**
 * PSK-31 Client - Main Application Entry
 *
 * This is the UI shell with mocked/simulated data for Phase 1.5.
 * Real backend integration happens in later phases.
 */

import { WaterfallDisplay } from './components/waterfall';
import { setupRxDisplay } from './components/rx-display';
import { setupTxInput } from './components/tx-input';
import { setupTxButtons } from './components/control-panel';
import { setupWaterfallClick } from './components/waterfall-controls';
import { setupThemeToggle } from './components/theme-toggle';
import { setupSerialPanel } from './components/serial-panel';
import { setupMenuEvents } from './services/event-handlers';

window.addEventListener('DOMContentLoaded', () => {
  const canvas = document.getElementById('waterfall-canvas') as HTMLCanvasElement;
  if (canvas) {
    const waterfall = new WaterfallDisplay(canvas);
    waterfall.start();
  }

  setupTxInput();
  setupTxButtons();
  setupRxDisplay();
  setupWaterfallClick();
  setupThemeToggle();
  setupSerialPanel();
  setupMenuEvents();
});
