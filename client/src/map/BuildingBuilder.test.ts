import { describe, it, expect } from 'vitest';
import { buildBuildings } from './BuildingBuilder';
import { Projection } from './Projection';
import type { BBox, Building } from './types';

const BBOX: BBox = {
  south: 51.51,
  west: -0.12,
  north: 51.52,
  east: -0.11,
};

function makeBuilding(id: number): Building {
  return {
    id,
    footprint: [
      { lat: 51.511, lon: -0.117 },
      { lat: 51.511, lon: -0.116 },
      { lat: 51.512, lon: -0.116 },
      { lat: 51.512, lon: -0.117 },
    ],
    height: 10,
  };
}

describe('BuildingBuilder', () => {
  it('builds a single merged mesh from buildings', () => {
    const proj = new Projection(BBOX);
    const mesh = buildBuildings([makeBuilding(1), makeBuilding(2)], proj);
    expect(mesh).not.toBeNull();
    expect(mesh!.name).toBe('buildings');
  });

  it('returns null for empty buildings', () => {
    const proj = new Projection(BBOX);
    const mesh = buildBuildings([], proj);
    expect(mesh).toBeNull();
  });

  it('single draw call (one mesh)', () => {
    const proj = new Projection(BBOX);
    const mesh = buildBuildings(
      [makeBuilding(1), makeBuilding(2), makeBuilding(3)],
      proj
    );
    expect(mesh).not.toBeNull();
    // It's a single Mesh, not a Group — one draw call
    expect(mesh!.type).toBe('Mesh');
  });

  it('mesh has geometry with vertices', () => {
    const proj = new Projection(BBOX);
    const mesh = buildBuildings([makeBuilding(1)], proj);
    expect(mesh).not.toBeNull();
    const pos = mesh!.geometry.getAttribute('position');
    expect(pos.count).toBeGreaterThan(0);
  });
});
