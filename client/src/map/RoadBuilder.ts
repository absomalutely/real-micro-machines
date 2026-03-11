import * as THREE from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import { Projection } from './Projection';
import type { Road } from './types';

/** Road Y position — slightly above ground to prevent z-fighting. */
const ROAD_Y = 0.05;

/** Color mapping by highway category. */
function roadColor(highwayType: string): number {
  switch (highwayType) {
    case 'motorway':
    case 'motorway_link':
      return 0x444444;
    case 'trunk':
    case 'trunk_link':
    case 'primary':
    case 'primary_link':
      return 0x555555;
    case 'secondary':
    case 'secondary_link':
    case 'tertiary':
    case 'tertiary_link':
      return 0x666666;
    case 'residential':
    case 'living_street':
    case 'unclassified':
      return 0x888888;
    case 'service':
      return 0x999999;
    case 'footway':
    case 'cycleway':
    case 'path':
    case 'pedestrian':
      return 0xaaaaaa;
    default:
      return 0x777777;
  }
}

/**
 * Build a triangle-strip geometry for a single road polyline.
 * Generates two triangles per segment with perpendicular width offsets.
 */
function buildRoadGeometry(
  points: { x: number; z: number }[],
  halfWidth: number
): THREE.BufferGeometry | null {
  if (points.length < 2) return null;

  const vertices: number[] = [];
  const normals: number[] = [];

  for (let i = 0; i < points.length; i++) {
    // Compute direction at this point (averaged for interior points)
    let dx = 0;
    let dz = 0;

    if (i === 0) {
      dx = points[1].x - points[0].x;
      dz = points[1].z - points[0].z;
    } else if (i === points.length - 1) {
      dx = points[i].x - points[i - 1].x;
      dz = points[i].z - points[i - 1].z;
    } else {
      // Average direction at junction for smooth corners
      const dx1 = points[i].x - points[i - 1].x;
      const dz1 = points[i].z - points[i - 1].z;
      const dx2 = points[i + 1].x - points[i].x;
      const dz2 = points[i + 1].z - points[i].z;
      dx = dx1 + dx2;
      dz = dz1 + dz2;
    }

    // Normalize
    const len = Math.sqrt(dx * dx + dz * dz);
    if (len < 1e-8) continue;
    dx /= len;
    dz /= len;

    // Perpendicular direction (rotate 90 degrees)
    const px = -dz * halfWidth;
    const pz = dx * halfWidth;

    // Left vertex
    vertices.push(points[i].x - px, ROAD_Y, points[i].z - pz);
    normals.push(0, 1, 0);

    // Right vertex
    vertices.push(points[i].x + px, ROAD_Y, points[i].z + pz);
    normals.push(0, 1, 0);
  }

  // Build triangle indices from the strip
  const numPoints = vertices.length / 3 / 2; // pairs of left/right
  if (numPoints < 2) return null;

  const indices: number[] = [];
  for (let i = 0; i < numPoints - 1; i++) {
    const bl = i * 2;
    const br = i * 2 + 1;
    const tl = (i + 1) * 2;
    const tr = (i + 1) * 2 + 1;
    // Two triangles per segment
    indices.push(bl, br, tl);
    indices.push(br, tr, tl);
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute(
    'position',
    new THREE.Float32BufferAttribute(vertices, 3)
  );
  geometry.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
  geometry.setIndex(indices);
  return geometry;
}

/**
 * Build all road meshes from map data.
 * Groups roads by type and merges geometries for draw-call optimization.
 */
export function buildRoads(roads: Road[], proj: Projection): THREE.Group {
  const group = new THREE.Group();
  group.name = 'roads';

  // Group roads by color
  const colorGroups = new Map<number, THREE.BufferGeometry[]>();

  for (const road of roads) {
    const projected = proj.projectAll(road.points);
    const halfWidth = road.width / 2;
    const geom = buildRoadGeometry(projected, halfWidth);
    if (!geom) continue;

    const color = roadColor(road.highway_type);
    if (!colorGroups.has(color)) {
      colorGroups.set(color, []);
    }
    colorGroups.get(color)!.push(geom);
  }

  // Merge and create meshes per color group
  for (const [color, geometries] of colorGroups) {
    if (geometries.length === 0) continue;

    const merged =
      geometries.length === 1 ? geometries[0] : mergeGeometries(geometries);
    if (!merged) continue;

    const material = new THREE.MeshStandardMaterial({
      color,
      roughness: 0.9,
      metalness: 0.0,
      side: THREE.DoubleSide,
    });

    const mesh = new THREE.Mesh(merged, material);
    mesh.name = `roads-${color.toString(16)}`;
    group.add(mesh);
  }

  return group;
}
