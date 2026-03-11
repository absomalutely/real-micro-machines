import { describe, it, expect } from 'vitest';
import * as THREE from 'three';
import { buildRoads } from './RoadBuilder';
import { Projection } from './Projection';
import type { BBox, Road } from './types';

const BBOX: BBox = {
  south: 51.51,
  west: -0.12,
  north: 51.52,
  east: -0.11,
};

function makeStraightRoad(): Road {
  return {
    id: 1,
    highway_type: 'residential',
    points: [
      { lat: 51.511, lon: -0.116 },
      { lat: 51.512, lon: -0.115 },
      { lat: 51.513, lon: -0.114 },
    ],
    width: 6,
    name: 'Test Road',
    oneway: false,
    lanes: 2,
  };
}

describe('RoadBuilder', () => {
  it('builds a group from roads', () => {
    const proj = new Projection(BBOX);
    const group = buildRoads([makeStraightRoad()], proj);
    expect(group.children.length).toBeGreaterThan(0);
  });

  it('returns empty group for no roads', () => {
    const proj = new Projection(BBOX);
    const group = buildRoads([], proj);
    expect(group.children.length).toBe(0);
  });

  it('road mesh has correct geometry structure', () => {
    const proj = new Projection(BBOX);
    const group = buildRoads([makeStraightRoad()], proj);
    const mesh = group.children[0] as THREE.Mesh;
    const geom = mesh.geometry;

    // 3 points → 2 segments → 4 triangles → 12 indices
    const index = geom.getIndex();
    expect(index).not.toBeNull();
    expect(index!.count).toBe(12); // 2 segments * 2 triangles * 3 vertices

    // 3 points → 6 vertices (left + right per point)
    const pos = geom.getAttribute('position');
    expect(pos.count).toBe(6);
  });

  it('roads with same type are merged', () => {
    const proj = new Projection(BBOX);
    const road1 = makeStraightRoad();
    const road2 = { ...makeStraightRoad(), id: 2, name: 'Another Road' };
    const group = buildRoads([road1, road2], proj);
    // Both residential → same color → merged into 1 mesh
    expect(group.children.length).toBe(1);
  });

  it('roads with different types are separate meshes', () => {
    const proj = new Projection(BBOX);
    const road1 = makeStraightRoad();
    const road2 = {
      ...makeStraightRoad(),
      id: 2,
      highway_type: 'motorway',
    };
    const group = buildRoads([road1, road2], proj);
    expect(group.children.length).toBe(2);
  });
});
