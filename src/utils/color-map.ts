/** Waterfall color map: dB magnitude → RGB color lookup table */

export type ColorPalette = 'classic' | 'heat' | 'viridis' | 'grayscale';

interface Stop { pos: number; r: number; g: number; b: number }

const PALETTE_STOPS: Record<ColorPalette, Stop[]> = {
  classic: [
    { pos: 0.00, r: 0,   g: 0,   b: 0   },
    { pos: 0.20, r: 0,   g: 0,   b: 170 },
    { pos: 0.40, r: 0,   g: 170, b: 170 },
    { pos: 0.55, r: 0,   g: 170, b: 0   },
    { pos: 0.70, r: 170, g: 170, b: 0   },
    { pos: 0.85, r: 255, g: 68,  b: 68  },
    { pos: 1.00, r: 255, g: 255, b: 255 },
  ],
  heat: [
    { pos: 0.00, r: 0,   g: 0,   b: 0   },
    { pos: 0.30, r: 139, g: 0,   b: 0   },
    { pos: 0.60, r: 255, g: 102, b: 0   },
    { pos: 0.80, r: 255, g: 204, b: 0   },
    { pos: 1.00, r: 255, g: 255, b: 255 },
  ],
  viridis: [
    { pos: 0.00, r: 68,  g: 1,   b: 84  },
    { pos: 0.25, r: 49,  g: 104, b: 142 },
    { pos: 0.50, r: 53,  g: 183, b: 121 },
    { pos: 0.75, r: 253, g: 231, b: 37  },
    { pos: 1.00, r: 240, g: 240, b: 240 },
  ],
  grayscale: [
    { pos: 0.0, r: 0,   g: 0,   b: 0   },
    { pos: 1.0, r: 255, g: 255, b: 255 },
  ],
};

function buildOne(stops: Stop[]): Uint8ClampedArray[] {
  const map: Uint8ClampedArray[] = [];
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
    map.push(color);
  }
  return map;
}

/** Build a single named palette's 256-entry LUT */
export function buildColorMap(palette: ColorPalette = 'classic'): Uint8ClampedArray[] {
  return buildOne(PALETTE_STOPS[palette]);
}

/** All valid palette names — single source of truth for validation */
export const VALID_PALETTES = Object.keys(PALETTE_STOPS) as ColorPalette[];

/** Build all palettes up front for zero-cost switching */
export function buildAllColorMaps(): Record<ColorPalette, Uint8ClampedArray[]> {
  return Object.fromEntries(
    VALID_PALETTES.map((k) => [k, buildOne(PALETTE_STOPS[k])]),
  ) as Record<ColorPalette, Uint8ClampedArray[]>;
}
