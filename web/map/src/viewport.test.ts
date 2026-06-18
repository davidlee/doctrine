import { describe, it, expect } from 'vitest';
import { fitViewport, applyFocusChange, clampViewport, parseTransform, readSvgDims, panViewport, type GraphViewport } from './viewport';

// ---------------------------------------------------------------------------
// fitViewport
// ---------------------------------------------------------------------------

describe('fitViewport', () => {
  it('returns k=1 and centred when SVG is smaller than container', () => {
    const vp = fitViewport(400, 300, 800, 600);
    expect(vp.k).toBe(1);
    // svgW=400, cw=800 → centre-x = (800 - 400)/2 = 200
    // svgH=300, ch=600 → centre-y = (600 - 300)/2 = 150
    expect(vp.x).toBeCloseTo(200, 5);
    expect(vp.y).toBeCloseTo(150, 5);
  });

  it('scales down to fit width when SVG is wider than container', () => {
    const vp = fitViewport(1600, 900, 800, 600);
    // k = min(800/1600, 600/900) = min(0.5, 0.667) = 0.5
    expect(vp.k).toBeCloseTo(0.5, 5);
    // x = (800 - 1600*0.5)/2 = (800 - 800)/2 = 0
    expect(vp.x).toBeCloseTo(0, 5);
    // y = (600 - 900*0.5)/2 = (600 - 450)/2 = 75
    expect(vp.y).toBeCloseTo(75, 5);
  });

  it('scales down to fit height when SVG is taller than container', () => {
    const vp = fitViewport(400, 1200, 800, 600);
    // k = min(800/400, 600/1200) = min(2, 0.5) = 0.5 — but k capped at 1
    expect(vp.k).toBeCloseTo(0.5, 5);
    // x = (800 - 400*0.5)/2 = (800 - 200)/2 = 300
    expect(vp.x).toBeCloseTo(300, 5);
    // y = (600 - 1200*0.5)/2 = (600 - 600)/2 = 0
    expect(vp.y).toBeCloseTo(0, 5);
  });

  it('does not upscale a SVG smaller than container (k capped at 1)', () => {
    const vp = fitViewport(200, 150, 800, 600);
    expect(vp.k).toBe(1);
    // x = (800 - 200*1)/2 = 300
    expect(vp.x).toBeCloseTo(300, 5);
    // y = (600 - 150*1)/2 = 225
    expect(vp.y).toBeCloseTo(225, 5);
  });

  it('guards against zero-width SVG', () => {
    const vp = fitViewport(0, 300, 800, 600);
    expect(vp).toEqual({ x: 0, y: 0, k: 1 });
  });

  it('guards against zero-height SVG', () => {
    const vp = fitViewport(400, 0, 800, 600);
    expect(vp).toEqual({ x: 0, y: 0, k: 1 });
  });

  it('guards against negative dimensions', () => {
    const vp = fitViewport(-100, -50, 800, 600);
    expect(vp).toEqual({ x: 0, y: 0, k: 1 });
  });
});

// ---------------------------------------------------------------------------
// applyFocusChange
// ---------------------------------------------------------------------------

describe('applyFocusChange', () => {
  const cw = 800;
  const ch = 600;

  it('returns fit-viewport when prior viewport is null (first render)', () => {
    const vp = applyFocusChange(null, 1, 400, 300, cw, ch);
    expect(vp.k).toBe(1);
    expect(vp.x).toBeCloseTo(200, 5);
    expect(vp.y).toBeCloseTo(150, 5);
  });

  it('preserves k and centres when prior k >= minK', () => {
    const prior: GraphViewport = { x: 100, y: 50, k: 2 };
    const vp = applyFocusChange(prior, 0.5, 800, 600, cw, ch);
    expect(vp.k).toBe(2);
    // centre at k=2: x = (800 - 800*2)/2 = (800 - 1600)/2 = -400
    expect(vp.x).toBeCloseTo(-400, 5);
    // centre at k=2: y = (600 - 600*2)/2 = (600 - 1200)/2 = -300
    expect(vp.y).toBeCloseTo(-300, 5);
  });

  it('clamps k to minK when prior k < minK and centres', () => {
    const prior: GraphViewport = { x: 100, y: 50, k: 0.3 };
    const vp = applyFocusChange(prior, 0.5, 800, 600, cw, ch);
    expect(vp.k).toBe(0.5);
    // centre at k=0.5: x = (800 - 800*0.5)/2 = (800 - 400)/2 = 200
    expect(vp.x).toBeCloseTo(200, 5);
    expect(vp.y).toBeCloseTo(150, 5);
  });

  it('preserves k when prior k equals minK exactly', () => {
    const prior: GraphViewport = { x: 10, y: 20, k: 0.75 };
    const vp = applyFocusChange(prior, 0.75, 1000, 500, cw, ch);
    expect(vp.k).toBe(0.75);
    // centre at k=0.75: x = (800 - 1000*0.75)/2 = (800 - 750)/2 = 25
    expect(vp.x).toBeCloseTo(25, 5);
    // centre at k=0.75: y = (600 - 500*0.75)/2 = (600 - 375)/2 = 112.5
    expect(vp.y).toBeCloseTo(112.5, 5);
  });
});

// ---------------------------------------------------------------------------
// clampViewport
// ---------------------------------------------------------------------------

describe('clampViewport', () => {
  it('returns unchanged viewport when k is within bounds', () => {
    const vp: GraphViewport = { x: 10, y: 20, k: 1.5 };
    const result = clampViewport(vp, 0.5, 3);
    expect(result).toEqual({ x: 10, y: 20, k: 1.5 });
  });

  it('clamps k to minK when below', () => {
    const vp: GraphViewport = { x: 10, y: 20, k: 0.1 };
    const result = clampViewport(vp, 0.5, 10);
    expect(result).toEqual({ x: 10, y: 20, k: 0.5 });
  });

  it('clamps k to maxK when above', () => {
    const vp: GraphViewport = { x: 10, y: 20, k: 15 };
    const result = clampViewport(vp, 0.5, 10);
    expect(result).toEqual({ x: 10, y: 20, k: 10 });
  });

  it('returns a new object (does not mutate input)', () => {
    const vp: GraphViewport = { x: 10, y: 20, k: 1 };
    const result = clampViewport(vp, 0.5, 10);
    expect(result).not.toBe(vp);
    expect(vp.k).toBe(1); // original unchanged
  });
});

// ---------------------------------------------------------------------------
// readSvgDims
// ---------------------------------------------------------------------------

describe('readSvgDims', () => {
  it('reads width and height from getBoundingClientRect', () => {
    const mockEl = {
      getBoundingClientRect: () => ({ width: 640, height: 480 }),
    } as unknown as SVGSVGElement;
    expect(readSvgDims(mockEl)).toEqual({ w: 640, h: 480 });
  });

  it('guards zero width', () => {
    const mockEl = {
      getBoundingClientRect: () => ({ width: 0, height: 480 }),
    } as unknown as SVGSVGElement;
    expect(readSvgDims(mockEl)).toEqual({ w: 1, h: 1 });
  });

  it('guards zero height', () => {
    const mockEl = {
      getBoundingClientRect: () => ({ width: 640, height: 0 }),
    } as unknown as SVGSVGElement;
    expect(readSvgDims(mockEl)).toEqual({ w: 1, h: 1 });
  });

  it('guards both zero', () => {
    const mockEl = {
      getBoundingClientRect: () => ({ width: 0, height: 0 }),
    } as unknown as SVGSVGElement;
    expect(readSvgDims(mockEl)).toEqual({ w: 1, h: 1 });
  });
});

// ---------------------------------------------------------------------------
// parseTransform
// ---------------------------------------------------------------------------

describe('parseTransform', () => {
  it('parses the format we write: translate(Xpx, Ypx) scale(K)', () => {
    expect(parseTransform('translate(100px, 200px) scale(1.5)')).toEqual({ x: 100, y: 200, k: 1.5 });
  });

  it('handles negative values', () => {
    expect(parseTransform('translate(-50.5px, 0px) scale(0.75)')).toEqual({ x: -50.5, y: 0, k: 0.75 });
  });

  it('handles precise scale values', () => {
    expect(parseTransform('translate(0px, 0px) scale(3.14159)')).toEqual({ x: 0, y: 0, k: 3.14159 });
  });

  it('returns identity on empty string', () => {
    expect(parseTransform('')).toEqual({ x: 0, y: 0, k: 1 });
  });

  it('returns identity on unrecognised format', () => {
    expect(parseTransform('matrix(1, 0, 0, 1, 0, 0)')).toEqual({ x: 0, y: 0, k: 1 });
  });

  it('handles translate without scale gracefully', () => {
    expect(parseTransform('translate(50px, 75px)')).toEqual({ x: 0, y: 0, k: 1 });
  });
});

// ---------------------------------------------------------------------------
// panViewport
// ---------------------------------------------------------------------------

describe('panViewport', () => {
  it('adds the screen-space delta to x/y and preserves k', () => {
    const vp: GraphViewport = { x: 10, y: 20, k: 1 };
    expect(panViewport(vp, 5, -7)).toEqual({ x: 15, y: 13, k: 1 });
  });

  it('does NOT scale the delta by k when zoomed out (regression: pan acceleration)', () => {
    // k < 1 (large graph fitted). A 100px mouse drag must move the layer 100px,
    // not 100/k. The old `/k` bug multiplied the delta when zoomed out.
    const vp: GraphViewport = { x: 0, y: 0, k: 0.5 };
    expect(panViewport(vp, 100, 0)).toEqual({ x: 100, y: 0, k: 0.5 });
  });

  it('does NOT shrink the delta by k when zoomed in', () => {
    const vp: GraphViewport = { x: 0, y: 0, k: 4 };
    expect(panViewport(vp, 100, 0)).toEqual({ x: 100, y: 0, k: 4 });
  });

  it('returns a new object, leaving the input untouched', () => {
    const vp: GraphViewport = { x: 1, y: 2, k: 3 };
    const out = panViewport(vp, 10, 10);
    expect(out).not.toBe(vp);
    expect(vp).toEqual({ x: 1, y: 2, k: 3 });
  });
});
