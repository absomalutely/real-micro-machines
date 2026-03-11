import * as THREE from 'three';
import { Projection } from './Projection';
import type { MapPolygon } from './types';

const WATER_COLOR = 0x4488cc;
const WATER_Y = -0.1;

/**
 * Build water polygons as flat semi-transparent planes.
 */
export function buildWater(
  water: MapPolygon[],
  proj: Projection
): THREE.Group {
  const group = new THREE.Group();
  group.name = 'water';

  for (const poly of water) {
    const projected = proj.projectAll(poly.points);
    if (projected.length < 3) continue;

    const shape = new THREE.Shape();
    shape.moveTo(projected[0].x, projected[0].z);
    for (let i = 1; i < projected.length; i++) {
      shape.lineTo(projected[i].x, projected[i].z);
    }
    shape.closePath();

    const geometry = new THREE.ShapeGeometry(shape);
    geometry.rotateX(-Math.PI / 2);

    const material = new THREE.MeshStandardMaterial({
      color: WATER_COLOR,
      transparent: true,
      opacity: 0.7,
      roughness: 0.3,
      metalness: 0.1,
      side: THREE.DoubleSide,
    });

    const mesh = new THREE.Mesh(geometry, material);
    mesh.position.y = WATER_Y;
    mesh.name = `water-${poly.id}`;
    group.add(mesh);
  }

  return group;
}
