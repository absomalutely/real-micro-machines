import * as THREE from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import { Projection } from './Projection';
import type { Building } from './types';

const BUILDING_COLOR = 0xc8b8a8; // Warm gray

/**
 * Build a single building geometry from its footprint and height.
 * Uses ExtrudeGeometry to extrude the footprint upward.
 */
function buildBuildingGeometry(
  footprint: { x: number; z: number }[],
  height: number
): THREE.BufferGeometry | null {
  if (footprint.length < 3) return null;

  // Create a 2D shape from the footprint (using X, Z as the shape coordinates)
  const shape = new THREE.Shape();
  shape.moveTo(footprint[0].x, footprint[0].z);
  for (let i = 1; i < footprint.length; i++) {
    shape.lineTo(footprint[i].x, footprint[i].z);
  }
  shape.closePath();

  // Check if the shape has non-zero area (avoid degenerate shapes)
  const points = shape.getPoints();
  if (points.length < 3) return null;

  // Calculate signed area to check for degenerate shapes
  let area = 0;
  for (let i = 0; i < points.length; i++) {
    const j = (i + 1) % points.length;
    area += points[i].x * points[j].y;
    area -= points[j].x * points[i].y;
  }
  if (Math.abs(area) < 0.01) return null;

  // Extrude upward
  const extrudeSettings: THREE.ExtrudeGeometryOptions = {
    depth: height,
    bevelEnabled: false,
  };

  const geometry = new THREE.ExtrudeGeometry(shape, extrudeSettings);

  // The ExtrudeGeometry extrudes along Z by default.
  // We need to rotate it so the extrusion goes along Y (up).
  // The shape is in XZ plane, extrusion goes along the shape's normal.
  // We created the shape in XZ, so extrusion goes along Y already IF
  // we rotate the geometry: swap Y and Z of the extrusion.
  geometry.rotateX(-Math.PI / 2);

  return geometry;
}

/**
 * Build all buildings as a single merged mesh for draw-call optimization.
 */
export function buildBuildings(
  buildings: Building[],
  proj: Projection
): THREE.Mesh | null {
  const geometries: THREE.BufferGeometry[] = [];

  for (const building of buildings) {
    const projected = proj.projectAll(building.footprint);
    const geom = buildBuildingGeometry(projected, building.height);
    if (geom) {
      geometries.push(geom);
    }
  }

  if (geometries.length === 0) return null;

  const merged =
    geometries.length === 1 ? geometries[0] : mergeGeometries(geometries);
  if (!merged) return null;

  const material = new THREE.MeshStandardMaterial({
    color: BUILDING_COLOR,
    roughness: 0.8,
    metalness: 0.1,
    side: THREE.DoubleSide,
  });

  const mesh = new THREE.Mesh(merged, material);
  mesh.name = 'buildings';
  return mesh;
}
