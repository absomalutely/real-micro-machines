import { describe, it, expect } from 'vitest';
import { Projection } from './Projection';
import type { BBox, LatLon } from './types';

const LONDON_BBOX: BBox = {
  south: 51.51,
  west: -0.12,
  north: 51.52,
  east: -0.11,
};

describe('Projection', () => {
  it('projects bbox center to (0, 0)', () => {
    const proj = new Projection(LONDON_BBOX);
    const center: LatLon = {
      lat: (LONDON_BBOX.south + LONDON_BBOX.north) / 2,
      lon: (LONDON_BBOX.west + LONDON_BBOX.east) / 2,
    };
    const { x, z } = proj.project(center);
    expect(Math.abs(x)).toBeLessThan(0.001);
    expect(Math.abs(z)).toBeLessThan(0.001);
  });

  it('east is positive X', () => {
    const proj = new Projection(LONDON_BBOX);
    const eastPoint: LatLon = { lat: 51.515, lon: -0.10 };
    const { x } = proj.project(eastPoint);
    expect(x).toBeGreaterThan(0);
  });

  it('north is negative Z (Three.js convention)', () => {
    const proj = new Projection(LONDON_BBOX);
    const northPoint: LatLon = { lat: 51.525, lon: -0.115 };
    const { z } = proj.project(northPoint);
    expect(z).toBeLessThan(0);
  });

  it('produces expected distances within 1m tolerance', () => {
    const proj = new Projection(LONDON_BBOX);
    // ~111m per 0.001 degrees latitude
    const p1: LatLon = { lat: 51.515, lon: -0.115 };
    const p2: LatLon = { lat: 51.516, lon: -0.115 };
    const r1 = proj.project(p1);
    const r2 = proj.project(p2);
    const dist = Math.abs(r2.z - r1.z);
    // 0.001 degrees latitude ≈ 111.3m
    expect(dist).toBeGreaterThan(110);
    expect(dist).toBeLessThan(113);
  });

  it('symmetry: equidistant north/south have equal |z|', () => {
    const proj = new Projection(LONDON_BBOX);
    const center = (LONDON_BBOX.south + LONDON_BBOX.north) / 2;
    const offset = 0.002;
    const north: LatLon = { lat: center + offset, lon: -0.115 };
    const south: LatLon = { lat: center - offset, lon: -0.115 };
    const rn = proj.project(north);
    const rs = proj.project(south);
    expect(Math.abs(Math.abs(rn.z) - Math.abs(rs.z))).toBeLessThan(0.01);
  });

  it('projectAll returns correct count', () => {
    const proj = new Projection(LONDON_BBOX);
    const points: LatLon[] = [
      { lat: 51.511, lon: -0.116 },
      { lat: 51.512, lon: -0.115 },
      { lat: 51.513, lon: -0.114 },
    ];
    const projected = proj.projectAll(points);
    expect(projected).toHaveLength(3);
  });

  it('bboxSizeMeters returns reasonable values for London', () => {
    const proj = new Projection(LONDON_BBOX);
    const { width, height } = proj.bboxSizeMeters(LONDON_BBOX);
    // ~0.01 degrees lat ≈ 1113m, ~0.01 degrees lon at 51.5° ≈ 695m
    expect(height).toBeGreaterThan(1000);
    expect(height).toBeLessThan(1200);
    expect(width).toBeGreaterThan(600);
    expect(width).toBeLessThan(800);
  });
});
