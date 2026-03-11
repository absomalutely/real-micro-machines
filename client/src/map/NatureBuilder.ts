import * as THREE from 'three';
import { Projection } from './Projection';
import type { MapPolygon } from './types';

const TREE_TRUNK_COLOR = 0x8b6914;
const TREE_CROWN_COLOR = 0x2d5a27;
const PARK_GROUND_COLOR = 0x5a8a4f;

/** Simple polygon bounds check using ray casting. */
function pointInPolygon(
  x: number,
  z: number,
  polygon: { x: number; z: number }[]
): boolean {
  let inside = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const xi = polygon[i].x;
    const zi = polygon[i].z;
    const xj = polygon[j].x;
    const zj = polygon[j].z;
    if (zi > z !== zj > z && x < ((xj - xi) * (z - zi)) / (zj - zi) + xi) {
      inside = !inside;
    }
  }
  return inside;
}

/** Get bounding box of projected polygon. */
function polygonBounds(polygon: { x: number; z: number }[]): {
  minX: number;
  maxX: number;
  minZ: number;
  maxZ: number;
} {
  let minX = Infinity,
    maxX = -Infinity;
  let minZ = Infinity,
    maxZ = -Infinity;
  for (const p of polygon) {
    minX = Math.min(minX, p.x);
    maxX = Math.max(maxX, p.x);
    minZ = Math.min(minZ, p.z);
    maxZ = Math.max(maxZ, p.z);
  }
  return { minX, maxX, minZ, maxZ };
}

/**
 * Simple Poisson-like placement using grid-based rejection sampling.
 * Returns positions within the polygon at roughly the given density.
 */
function samplePointsInPolygon(
  polygon: { x: number; z: number }[],
  spacing: number,
  maxPoints: number = 500
): { x: number; z: number }[] {
  const bounds = polygonBounds(polygon);
  const points: { x: number; z: number }[] = [];

  // Add jitter to grid for more natural placement
  for (let x = bounds.minX; x <= bounds.maxX; x += spacing) {
    for (let z = bounds.minZ; z <= bounds.maxZ; z += spacing) {
      if (points.length >= maxPoints) return points;

      // Add random jitter (±40% of spacing)
      const jitterX = (Math.random() - 0.5) * spacing * 0.8;
      const jitterZ = (Math.random() - 0.5) * spacing * 0.8;
      const px = x + jitterX;
      const pz = z + jitterZ;

      if (pointInPolygon(px, pz, polygon)) {
        points.push({ x: px, z: pz });
      }
    }
  }

  return points;
}

/**
 * Build trees as InstancedMesh for performance.
 * Each tree is a cone (trunk) + sphere (crown).
 */
function buildTrees(
  positions: { x: number; z: number }[]
): THREE.Group {
  const group = new THREE.Group();
  if (positions.length === 0) return group;

  // Tree trunk: thin cylinder
  const trunkGeom = new THREE.CylinderGeometry(0.3, 0.4, 2, 6);
  const trunkMat = new THREE.MeshStandardMaterial({
    color: TREE_TRUNK_COLOR,
    roughness: 0.9,
  });
  const trunks = new THREE.InstancedMesh(trunkGeom, trunkMat, positions.length);

  // Tree crown: cone
  const crownGeom = new THREE.ConeGeometry(2, 4, 6);
  const crownMat = new THREE.MeshStandardMaterial({
    color: TREE_CROWN_COLOR,
    roughness: 0.8,
  });
  const crowns = new THREE.InstancedMesh(crownGeom, crownMat, positions.length);

  const matrix = new THREE.Matrix4();

  for (let i = 0; i < positions.length; i++) {
    const { x, z } = positions[i];
    // Random scale variation (0.7 to 1.3)
    const scale = 0.7 + Math.random() * 0.6;

    // Trunk: positioned at base
    matrix.makeTranslation(x, 1 * scale, z);
    matrix.scale(new THREE.Vector3(scale, scale, scale));
    trunks.setMatrixAt(i, matrix);

    // Crown: positioned above trunk
    matrix.makeTranslation(x, 4 * scale, z);
    matrix.scale(new THREE.Vector3(scale, scale, scale));
    crowns.setMatrixAt(i, matrix);
  }

  trunks.instanceMatrix.needsUpdate = true;
  crowns.instanceMatrix.needsUpdate = true;
  trunks.name = 'tree-trunks';
  crowns.name = 'tree-crowns';

  group.add(trunks);
  group.add(crowns);
  return group;
}

/**
 * Build park/forest ground planes.
 */
function buildParkGround(
  polygon: { x: number; z: number }[]
): THREE.Mesh | null {
  if (polygon.length < 3) return null;

  // Negate z when feeding into Shape's Y to compensate for rotateX(-PI/2)
  // which maps shape-Y → world -Z (double-negation → correct world Z).
  const shape = new THREE.Shape();
  shape.moveTo(polygon[0].x, -polygon[0].z);
  for (let i = 1; i < polygon.length; i++) {
    shape.lineTo(polygon[i].x, -polygon[i].z);
  }
  shape.closePath();

  const geometry = new THREE.ShapeGeometry(shape);
  geometry.rotateX(-Math.PI / 2);

  const material = new THREE.MeshStandardMaterial({
    color: PARK_GROUND_COLOR,
    roughness: 0.95,
    side: THREE.DoubleSide,
  });

  const mesh = new THREE.Mesh(geometry, material);
  mesh.position.y = 0.01; // Just above terrain
  return mesh;
}

/**
 * Build nature elements (trees in parks/forests, ground planes).
 */
export function buildNature(
  parks: MapPolygon[],
  forests: MapPolygon[],
  proj: Projection
): THREE.Group {
  const group = new THREE.Group();
  group.name = 'nature';

  const allTreePositions: { x: number; z: number }[] = [];

  // Parks: lower tree density, with ground plane
  for (const park of parks) {
    const projected = proj.projectAll(park.points);
    if (projected.length < 3) continue;

    // Park ground
    const ground = buildParkGround(projected);
    if (ground) group.add(ground);

    // Trees in parks (1 per ~100m² → spacing ~10m)
    const positions = samplePointsInPolygon(projected, 10, 200);
    allTreePositions.push(...positions);
  }

  // Forests: higher tree density, no separate ground
  for (const forest of forests) {
    const projected = proj.projectAll(forest.points);
    if (projected.length < 3) continue;

    // Trees in forests (1 per ~50m² → spacing ~7m)
    const positions = samplePointsInPolygon(projected, 7, 500);
    allTreePositions.push(...positions);
  }

  // Build all trees as one instanced group
  if (allTreePositions.length > 0) {
    const trees = buildTrees(allTreePositions);
    trees.name = 'trees';
    group.add(trees);
  }

  return group;
}
