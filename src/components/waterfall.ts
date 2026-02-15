/** Waterfall spectrum display - canvas-based scrolling spectrogram */

import { buildColorMap } from '../utils/color-map';

export class WaterfallDisplay {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private imageData: ImageData | null = null;
  private animationId: number = 0;
  private colorMap: Uint8ClampedArray[] = [];
  private resizeHandler = () => this.resize();
  private liveMode: boolean = false;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d', { alpha: false })!;
    this.colorMap = buildColorMap();
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

  /**
   * Draw a single spectrum line from real FFT magnitudes (in dB).
   * Called by the audio bridge when an fft-data event arrives.
   *
   * Maps the FFT bins covering 500-2500 Hz to the canvas width.
   * dB normalization: (dbValue + 100) / 80 * 255 → 0-255 color index
   */
  drawSpectrum(magnitudes: number[]): void {
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

    // Map FFT bins to the 500-2500 Hz display range
    // At 48 kHz sample rate with 4096-point FFT, each bin = 48000/4096 ≈ 11.72 Hz
    // Bin index for freq f = f * fftSize / sampleRate
    const sampleRate = 48000;
    const fftSize = magnitudes.length * 2; // magnitudes is half the FFT size
    const binWidth = sampleRate / fftSize;
    const startBin = Math.floor(500 / binWidth);
    const endBin = Math.ceil(2500 / binWidth);
    const displayBins = endBin - startBin;

    // Draw the new top row
    for (let x = 0; x < width; x++) {
      // Map pixel to FFT bin (linear interpolation)
      const binFloat = startBin + (x / width) * displayBins;
      const binIdx = Math.floor(binFloat);

      // Clamp to valid range
      const dbValue = binIdx < magnitudes.length ? magnitudes[binIdx] : -100;

      // Normalize dB to 0-255 color index
      // Typical range: -100 dB (silence) to -20 dB (strong signal)
      const normalized = Math.min(255, Math.max(0, Math.floor(((dbValue + 100) / 80) * 255)));
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
