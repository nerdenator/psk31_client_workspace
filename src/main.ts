/**
 * PSK-31 Client - Main Application Entry
 *
 * This is the UI shell with mocked/simulated data for Phase 1.5.
 * Real backend integration happens in later phases.
 */

import { listen } from '@tauri-apps/api/event';

// Menu event payload from Rust
interface MenuEvent {
  id: string;
}

// Waterfall simulation
class WaterfallDisplay {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private imageData: ImageData | null = null;
  private animationId: number = 0;
  private colorMap: Uint8ClampedArray[] = [];

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d', { alpha: false })!;
    this.buildColorMap();
    this.resize();
    window.addEventListener('resize', () => this.resize());
  }

  private buildColorMap(): void {
    // Build a 256-entry color lookup table (black -> blue -> cyan -> green -> yellow -> red -> white)
    const stops = [
      { pos: 0, r: 0, g: 0, b: 0 },
      { pos: 0.2, r: 0, g: 0, b: 170 },
      { pos: 0.4, r: 0, g: 170, b: 170 },
      { pos: 0.55, r: 0, g: 170, b: 0 },
      { pos: 0.7, r: 170, g: 170, b: 0 },
      { pos: 0.85, r: 255, g: 68, b: 68 },
      { pos: 1.0, r: 255, g: 255, b: 255 }
    ];

    for (let i = 0; i < 256; i++) {
      const pos = i / 255;
      let color = new Uint8ClampedArray([0, 0, 0, 255]);

      for (let j = 0; j < stops.length - 1; j++) {
        if (pos >= stops[j].pos && pos <= stops[j + 1].pos) {
          const t = (pos - stops[j].pos) / (stops[j + 1].pos - stops[j].pos);
          color[0] = Math.round(stops[j].r + t * (stops[j + 1].r - stops[j].r));
          color[1] = Math.round(stops[j].g + t * (stops[j + 1].g - stops[j].g));
          color[2] = Math.round(stops[j].b + t * (stops[j + 1].b - stops[j].b));
          break;
        }
      }
      this.colorMap.push(color);
    }
  }

  private resize(): void {
    const rect = this.canvas.parentElement!.getBoundingClientRect();
    this.canvas.width = rect.width;
    this.canvas.height = rect.height;
    this.imageData = this.ctx.createImageData(this.canvas.width, this.canvas.height);
    // Fill with black
    for (let i = 3; i < this.imageData.data.length; i += 4) {
      this.imageData.data[i] = 255;
    }
  }

  start(): void {
    const animate = () => {
      this.drawFrame();
      this.animationId = requestAnimationFrame(animate);
    };
    animate();
  }

  stop(): void {
    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
    }
  }

  private drawFrame(): void {
    if (!this.imageData) return;

    const width = this.canvas.width;
    const height = this.canvas.height;
    const data = this.imageData.data;

    // Scroll existing data down by 1 row
    for (let y = height - 1; y > 0; y--) {
      for (let x = 0; x < width; x++) {
        const srcIdx = ((y - 1) * width + x) * 4;
        const dstIdx = (y * width + x) * 4;
        data[dstIdx] = data[srcIdx];
        data[dstIdx + 1] = data[srcIdx + 1];
        data[dstIdx + 2] = data[srcIdx + 2];
      }
    }

    // Generate new top row with simulated spectrum
    const freqRange = 2000;  // 500-2500 Hz displayed
    const noiseFloor = 15;
    const time = performance.now() / 1000;

    for (let x = 0; x < width; x++) {
      // Map pixel to frequency
      const freq = 500 + (x / width) * freqRange;

      // Base noise
      let magnitude = Math.random() * noiseFloor;

      // Simulated signal at center frequency (1500 Hz) - PSK31 signal
      const signalCenter = 1500;
      const signalWidth = 60;
      const distFromSignal = Math.abs(freq - signalCenter);
      if (distFromSignal < signalWidth) {
        const signalStrength = 1 - (distFromSignal / signalWidth);
        // Add some modulation to simulate PSK
        const modulation = Math.sin(time * 20 + x * 0.1) * 0.3 + 0.7;
        magnitude += signalStrength * 200 * modulation * (Math.random() * 0.3 + 0.7);
      }

      // Add a weaker signal at 1200 Hz
      const signal2Center = 1200;
      const dist2 = Math.abs(freq - signal2Center);
      if (dist2 < 40) {
        const strength = (1 - dist2 / 40) * 100 * (Math.random() * 0.4 + 0.6);
        magnitude += strength * (Math.sin(time * 15) * 0.3 + 0.7);
      }

      // Add occasional burst at 1800 Hz
      if (Math.sin(time * 0.5) > 0.7) {
        const dist3 = Math.abs(freq - 1800);
        if (dist3 < 30) {
          magnitude += (1 - dist3 / 30) * 80;
        }
      }

      // Clamp and map to color
      magnitude = Math.min(255, Math.max(0, magnitude));
      const color = this.colorMap[Math.floor(magnitude)];

      const idx = x * 4;
      data[idx] = color[0];
      data[idx + 1] = color[1];
      data[idx + 2] = color[2];
    }

    this.ctx.putImageData(this.imageData, 0, 0);
  }
}

// TX character counter
function setupTxInput(): void {
  const txInput = document.getElementById('tx-input') as HTMLTextAreaElement;
  const charCount = document.querySelector('.tx-char-count .current') as HTMLElement;

  if (txInput && charCount) {
    txInput.addEventListener('input', () => {
      charCount.textContent = txInput.value.length.toString();
    });
  }
}

// TX button states (mock)
function setupTxButtons(): void {
  const sendBtn = document.querySelector('.tx-btn-send') as HTMLButtonElement;
  const abortBtn = document.querySelector('.tx-btn-abort') as HTMLButtonElement;
  const txIndicator = document.querySelector('.tx-indicator') as HTMLElement;
  const pttIndicator = document.querySelector('.ptt-indicator') as HTMLElement;
  const pttStatus = document.querySelector('.ptt-status') as HTMLElement;
  const txInput = document.getElementById('tx-input') as HTMLTextAreaElement;

  if (sendBtn && abortBtn) {
    sendBtn.addEventListener('click', () => {
      if (txInput.value.trim() === '') return;

      // Simulate TX state
      txIndicator.classList.add('active');
      pttIndicator.classList.remove('rx');
      pttIndicator.classList.add('tx');
      pttIndicator.textContent = 'TX';
      pttStatus.textContent = 'Transmitting';
      sendBtn.disabled = true;
      abortBtn.disabled = false;
      txInput.disabled = true;

      // Auto-return to RX after simulated transmission
      setTimeout(() => {
        resetToRx();
      }, 3000);
    });

    abortBtn.addEventListener('click', () => {
      resetToRx();
    });

    function resetToRx(): void {
      txIndicator.classList.remove('active');
      pttIndicator.classList.remove('tx');
      pttIndicator.classList.add('rx');
      pttIndicator.textContent = 'RX';
      pttStatus.textContent = 'Receiving';
      sendBtn.disabled = false;
      abortBtn.disabled = true;
      txInput.disabled = false;
    }
  }
}

// RX clear button
function setupRxControls(): void {
  const clearBtn = document.querySelector('.rx-controls .rx-btn') as HTMLButtonElement;
  const rxContent = document.getElementById('rx-content') as HTMLElement;

  if (clearBtn && rxContent) {
    clearBtn.addEventListener('click', () => {
      rxContent.textContent = '';
    });
  }
}

// Waterfall click-to-tune (mock)
function setupWaterfallClick(): void {
  const canvas = document.getElementById('waterfall-canvas') as HTMLCanvasElement;
  const freqDisplay = document.querySelector('.waterfall-freq') as HTMLElement;
  const carrierMarker = document.querySelector('.carrier-marker') as HTMLElement;
  const statusCarrier = document.querySelector('.status-item .value.highlight') as HTMLElement;

  if (canvas) {
    canvas.addEventListener('click', (e) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const freq = Math.round(500 + (x / rect.width) * 2000);

      // Update displays
      freqDisplay.textContent = `${freq} Hz`;
      if (statusCarrier) statusCarrier.textContent = `${freq} Hz`;

      // Update carrier marker position
      const markerX = ((freq - 500) / 2000) * 100;
      carrierMarker.style.left = `${markerX}%`;

      // Update sidebar audio carrier display
      const audioCarrierValue = document.querySelector('.sidebar-section:nth-child(3) .frequency-value') as HTMLElement;
      if (audioCarrierValue) {
        audioCarrierValue.textContent = freq.toString();
      }
    });
  }
}

// Set theme and update UI
function setTheme(theme: 'light' | 'dark'): void {
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.setItem('psk31-theme', theme);

  const icon = document.querySelector('.theme-toggle-icon') as HTMLElement;
  if (icon) {
    icon.textContent = theme === 'dark' ? '☀' : '☽';
  }
}

// Get current theme
function getCurrentTheme(): 'light' | 'dark' {
  return (document.documentElement.getAttribute('data-theme') as 'light' | 'dark') || 'light';
}

// Theme toggle
function setupThemeToggle(): void {
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

// Menu event handlers
function setupMenuEvents(): void {
  listen<MenuEvent>('menu-event', (event) => {
    const { id } = event.payload;
    console.log(`Menu event: ${id}`);

    switch (id) {
      case 'settings':
        alert('Settings dialog coming soon');
        break;

      case 'config_default':
        console.log('Switched to Default configuration');
        break;

      case 'config_save':
        alert('Save Configuration coming soon');
        break;

      case 'config_delete':
        alert('Delete Configuration coming soon');
        break;

      case 'theme_light':
        setTheme('light');
        break;

      case 'theme_dark':
        setTheme('dark');
        break;

      case 'waterfall_colors':
        alert('Waterfall Colors coming soon');
        break;

      case 'zoom_in':
        console.log('Zoom in');
        break;

      case 'zoom_out':
        console.log('Zoom out');
        break;

      case 'zoom_reset':
        console.log('Zoom reset');
        break;

      case 'documentation':
        window.open('https://github.com/nerdenator/psk31_client_workspace', '_blank');
        break;

      case 'about':
        alert('PSK-31 Client v0.1.0\n\nA cross-platform desktop application for PSK-31 ham radio communication.');
        break;

      default:
        console.log(`Unhandled menu event: ${id}`);
    }
  });
}

// Initialize
window.addEventListener('DOMContentLoaded', () => {
  const canvas = document.getElementById('waterfall-canvas') as HTMLCanvasElement;

  if (canvas) {
    const waterfall = new WaterfallDisplay(canvas);
    waterfall.start();
  }

  setupTxInput();
  setupTxButtons();
  setupRxControls();
  setupWaterfallClick();
  setupThemeToggle();
  setupMenuEvents();
});
