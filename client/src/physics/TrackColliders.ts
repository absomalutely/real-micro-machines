import { Projection } from '../map/Projection';
import type { Building, MapData, Road } from '../map/types';
import { RAPIER, type PhysicsWorld } from './PhysicsWorld';
import type { SurfaceType } from './surfaces';

// Handle → surface type mapping
const colliderSurfaceMap = new Map<number, SurfaceType>();

export function getSurfaceForCollider(collider: RAPIER.Collider): SurfaceType {
  return colliderSurfaceMap.get(collider.handle) ?? 'grass';
}

function registerSurface(collider: RAPIER.Collider, surface: SurfaceType): void {
  colliderSurfaceMap.set(collider.handle, surface);
}

export function buildTrackColliders(
  mapData: MapData,
  projection: Projection,
  world: PhysicsWorld,
): void {
  // Reset surface map
  colliderSurfaceMap.clear();

  // Roads — thin cuboid strips along each road segment
  for (const road of mapData.roads) {
    buildRoadColliders(road, projection, world);
  }

  // Buildings — AABB cuboids
  for (const building of mapData.buildings) {
    buildBuildingCollider(building, projection, world);
  }

  // Water — sensor colliders (no physics response)
  for (const water of mapData.water) {
    buildWaterCollider(water, projection, world);
  }
}

function buildRoadColliders(road: Road, projection: Projection, world: PhysicsWorld): void {
  if (road.points.length < 2) return;

  const projected = projection.projectAll(road.points);
  const halfWidth = road.width / 2;
  const halfHeight = 0.05; // Very thin

  // Create a cuboid for each road segment
  for (let i = 0; i < projected.length - 1; i++) {
    const p0 = projected[i];
    const p1 = projected[i + 1];

    const dx = p1.x - p0.x;
    const dz = p1.z - p0.z;
    const segLen = Math.sqrt(dx * dx + dz * dz);
    if (segLen < 0.1) continue;

    const halfLen = segLen / 2;
    const cx = (p0.x + p1.x) / 2;
    const cz = (p0.z + p1.z) / 2;
    const angle = Math.atan2(dx, dz);

    const bodyDesc = RAPIER.RigidBodyDesc.fixed()
      .setTranslation(cx, 0.05, cz)
      .setRotation({ x: 0, y: Math.sin(angle / 2), z: 0, w: Math.cos(angle / 2) });

    const body = world.createRigidBody(bodyDesc);
    const colliderDesc = RAPIER.ColliderDesc.cuboid(halfWidth, halfHeight, halfLen)
      .setFriction(0.8);
    const collider = world.createCollider(colliderDesc, body);
    registerSurface(collider, 'road');
  }
}

function buildBuildingCollider(building: Building, projection: Projection, world: PhysicsWorld): void {
  if (building.footprint.length < 3) return;

  const projected = projection.projectAll(building.footprint);

  // Compute AABB
  let minX = Infinity, maxX = -Infinity;
  let minZ = Infinity, maxZ = -Infinity;
  for (const p of projected) {
    if (p.x < minX) minX = p.x;
    if (p.x > maxX) maxX = p.x;
    if (p.z < minZ) minZ = p.z;
    if (p.z > maxZ) maxZ = p.z;
  }

  const halfW = (maxX - minX) / 2;
  const halfD = (maxZ - minZ) / 2;
  const halfH = building.height / 2;

  if (halfW < 0.1 || halfD < 0.1) return;

  const cx = (minX + maxX) / 2;
  const cz = (minZ + maxZ) / 2;

  const bodyDesc = RAPIER.RigidBodyDesc.fixed()
    .setTranslation(cx, halfH, cz);
  const body = world.createRigidBody(bodyDesc);
  const colliderDesc = RAPIER.ColliderDesc.cuboid(halfW, halfH, halfD)
    .setFriction(0.3)
    .setRestitution(0.5);
  const collider = world.createCollider(colliderDesc, body);
  registerSurface(collider, 'building');
}

function buildWaterCollider(water: { points: { lat: number; lon: number }[] }, projection: Projection, world: PhysicsWorld): void {
  if (water.points.length < 3) return;

  const projected = projection.projectAll(water.points);

  // AABB for water region
  let minX = Infinity, maxX = -Infinity;
  let minZ = Infinity, maxZ = -Infinity;
  for (const p of projected) {
    if (p.x < minX) minX = p.x;
    if (p.x > maxX) maxX = p.x;
    if (p.z < minZ) minZ = p.z;
    if (p.z > maxZ) maxZ = p.z;
  }

  const halfW = (maxX - minX) / 2;
  const halfD = (maxZ - minZ) / 2;
  if (halfW < 0.1 || halfD < 0.1) return;

  const cx = (minX + maxX) / 2;
  const cz = (minZ + maxZ) / 2;

  const bodyDesc = RAPIER.RigidBodyDesc.fixed()
    .setTranslation(cx, -0.1, cz);
  const body = world.createRigidBody(bodyDesc);
  const colliderDesc = RAPIER.ColliderDesc.cuboid(halfW, 0.5, halfD)
    .setSensor(true);
  const collider = world.createCollider(colliderDesc, body);
  registerSurface(collider, 'water');
}
