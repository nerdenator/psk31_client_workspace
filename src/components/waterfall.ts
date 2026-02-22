/** Waterfall spectrum display - canvas-based scrolling spectrogram */

import { buildAllColorMaps, type ColorPalette } from '../utils/color-map';

export type ZoomLevel = 1 | 2 | 4;

export interface WaterfallSettings {
  palette: ColorPalette;
  noiseFloor: number;
  zoomLevel: ZoomLevel;
}

const AUDIO_START_HZ = 500;
const AUDIO_END_HZ = 2500;
const SAMPLE_RATE = 48000;

export class WaterfallDisplay {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private imageData: ImageData | null = null;
  private animationId: number = 0;
  private allColorMaps = buildAllColorMaps();
  private colorMap: Uint8ClampedArray[];
  private resizeHandler = () => this.resize();
  private liveMode: boolean = false;

  // Adjustable settings
  private palette: ColorPalette = 'classic';
  private noiseFloor: number = -100;
  private readonly dynamicRange: number = 80;
  private zoomLevel: ZoomLevel = 1;
  private carrierFreq: number = 1500;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d', { alpha: false })!;
    this.colorMap = this.allColorMaps.classic;
    this.resize();
    window.addEventListener('resize', this.resizeHandler);
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
      if (!this.liveMode) {
        this.drawFrame();
      }
      this.animationId = requestAnimationFrame(animate);
    };
    animate();
  }

  stop(): void {
    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
    }
    window.removeEventListener('resize', this.resizeHandler);
  }

  /** Switch between simulated and live FFT data */
  setLiveMode(live: boolean): void {
    this.liveMode = live;
  }

  // --- Settings ---

  setPalette(palette: ColorPalette): void {
    this.palette = palette;
    this.colorMap = this.allColorMaps[palette];
  }

  setNoiseFloor(dbValue: number): void {
    this.noiseFloor = dbValue;
  }

  setZoom(level: ZoomLevel): void {
    this.zoomLevel = level;
  }

  /** Called by click-to-tune so zoom stays centered on the active carrier */
  setCarrierFreq(freqHz: number): void {
    this.carrierFreq = freqHz;
  }

  getSettings(): WaterfallSettings {
    return {
      palette: this.palette,
      noiseFloor: this.noiseFloor,
      zoomLevel: this.zoomLevel,
    };
  }

  /** Returns the currently visible Hz range based on zoom + carrier */
  getVisibleRange(): { startHz: number; endHz: number } {
    if (this.zoomLevel === 1) {
      return { startHz: AUDIO_START_HZ, endHz: AUDIO_END_HZ };
    }
    const span = (AUDIO_END_HZ - AUDIO_START_HZ) / this.zoomLevel; // 1000 or 500
    const half = span / 2;
    const startHz = Math.max(
      AUDIO_START_HZ,
      Math.min(AUDIO_END_HZ - span, this.carrierFreq - half),
    );
    return { startHz, endHz: startHz + span };
  }

  private scrollDown(): void {
    if (!this.imageData) return;
    const { width, height } = this.canvas;
    const data = this.imageData.data;
    for (let y = height - 1; y > 0; y--) {
      for (let x = 0; x < width; x++) {
        const srcIdx = ((y - 1) * width + x) * 4;
        const dstIdx = (y * width + x) * 4;
        data[dstIdx] = data[srcIdx];
        data[dstIdx + 1] = data[srcIdx + 1];
        data[dstIdx + 2] = data[srcIdx + 2];
      }
    }
  }

  /**
   * Draw a single spectrum line from real FFT magnitudes (in dB).
   * Called by the audio bridge when an fft-data event arrives.
   */
  drawSpectrum(magnitudes: number[]): void {
    if (!this.imageData) return;

    this.scrollDown();

    // Map FFT bins to the visible Hz range (zoom + carrier aware)
    const { width } = this.canvas;
    const data = this.imageData.data;
    const fftSize = magnitudes.length * 2;
    const binWidth = SAMPLE_RATE / fftSize;
    const { startHz, endHz } = this.getVisibleRange();
    const startBin = Math.floor(startHz / binWidth);
    const endBin = Math.ceil(endHz / binWidth);
    const displayBins = endBin - startBin;

    // Draw the new top row
    for (let x = 0; x < width; x++) {
      const binFloat = startBin + (x / width) * displayBins;
      const binIdx = Math.floor(binFloat);
      const dbValue = binIdx < magnitudes.length ? magnitudes[binIdx] : this.noiseFloor;

      // Normalize dB to 0-255 using adjustable noise floor
      const normalized = Math.min(
        255,
        Math.max(0, Math.floor(((dbValue - this.noiseFloor) / this.dynamicRange) * 255)),
      );
      const color = this.colorMap[normalized];

      const idx = x * 4;
      data[idx] = color[0];
      data[idx + 1] = color[1];
      data[idx + 2] = color[2];
    }

    this.ctx.putImageData(this.imageData, 0, 0);
  }

  private drawFrame(): void {
    if (!this.imageData) return;

    this.scrollDown();

    // Generate new top row with simulated spectrum (raw 0-255 magnitudes)
    const { width } = this.canvas;
    const data = this.imageData.data;
    const { startHz, endHz } = this.getVisibleRange();
    const freqRange = endHz - startHz;
    const noiseFloor = 15;
    const time = performance.now() / 1000;

    for (let x = 0; x < width; x++) {
      const freq = startHz + (x / width) * freqRange;

      let magnitude = Math.random() * noiseFloor;

      // Simulated PSK-31 signal at center frequency (1500 Hz)
      const distFromSignal = Math.abs(freq - 1500);
      if (distFromSignal < 60) {
        const signalStrength = 1 - distFromSignal / 60;
        const modulation = Math.sin(time * 20 + x * 0.1) * 0.3 + 0.7;
        magnitude += signalStrength * 200 * modulation * (Math.random() * 0.3 + 0.7);
      }

      // Weaker signal at 1200 Hz
      const dist2 = Math.abs(freq - 1200);
      if (dist2 < 40) {
        const strength = (1 - dist2 / 40) * 100 * (Math.random() * 0.4 + 0.6);
        magnitude += strength * (Math.sin(time * 15) * 0.3 + 0.7);
      }

      // Occasional burst at 1800 Hz
      if (Math.sin(time * 0.5) > 0.7) {
        const dist3 = Math.abs(freq - 1800);
        if (dist3 < 30) {
          magnitude += (1 - dist3 / 30) * 80;
        }
      }

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
