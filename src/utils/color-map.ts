/** Waterfall color map: dB magnitude → RGB color lookup table */

export function buildColorMap(): Uint8ClampedArray[] {
  const stops = [
    { pos: 0, r: 0, g: 0, b: 0 },
    { pos: 0.2, r: 0, g: 0, b: 170 },
    { pos: 0.4, r: 0, g: 170, b: 170 },
    { pos: 0.55, r: 0, g: 170, b: 0 },
    { pos: 0.7, r: 170, g: 170, b: 0 },
    { pos: 0.85, r: 255, g: 68, b: 68 },
    { pos: 1.0, r: 255, g: 255, b: 255 }
  ];

  const colorMap: Uint8ClampedArray[] = [];

  for (let i = 0; i < 256; i++) {
    const pos = i / 255;
    const color = new Uint8ClampedArray([0, 0, 0, 255]);

    for (let j = 0; j < stops.length - 1; j++) {
      if (pos >= stops[j].pos && pos <= stops[j + 1].pos) {
        const t = (pos - stops[j].pos) / (stops[j + 1].pos - stops[j].pos);
        color[0] = Math.round(stops[j].r + t * (stops[j + 1].r - stops[j].r));
        color[1] = Math.round(stops[j].g + t * (stops[j + 1].g - stops[j].g));
        color[2] = Math.round(stops[j].b + t * (stops[j + 1].b - stops[j].b));
        break;
      }
    }
    colorMap.push(color);
  }

  return colorMap;
}
