/** Waterfall click-to-tune interaction */

export function setupWaterfallClick(): void {
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
