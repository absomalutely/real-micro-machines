import * as THREE from 'three';
import { Projection } from './Projection';
import type { BBox } from './types';

const TERRAIN_COLOR = 0x4a7c3f;

/**
 * Build a flat green ground plane covering the bbox area.
 * Extends 10% beyond bbox to avoid visible edges.
 */
export function buildTerrain(bbox: BBox, proj: Projection): THREE.Mesh {
  const { width, height } = proj.bboxSizeMeters(bbox);

  // Add 10% padding on each side
  const paddedWidth = width * 1.2;
  const paddedHeight = height * 1.2;

  const geometry = new THREE.PlaneGeometry(paddedWidth, paddedHeight);
  geometry.rotateX(-Math.PI / 2); // Lay flat (XZ plane)

  const material = new THREE.MeshStandardMaterial({
    color: TERRAIN_COLOR,
    roughness: 0.95,
    metalness: 0.0,
    side: THREE.DoubleSide,
  });

  const mesh = new THREE.Mesh(geometry, material);
  mesh.position.y = 0;
  mesh.name = 'terrain';
  mesh.receiveShadow = true;

  return mesh;
}
