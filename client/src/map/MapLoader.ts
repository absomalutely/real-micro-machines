import { unpack } from 'msgpackr';
import * as THREE from 'three';
import { buildBuildings } from './BuildingBuilder';
import { buildNature } from './NatureBuilder';
import { Projection } from './Projection';
import { buildRoads } from './RoadBuilder';
import { buildTerrain } from './TerrainBuilder';
import type { BBox, MapData, TrackData } from './types';
import { buildWater } from './WaterBuilder';

/**
 * Fetch map data from the server's /api/map endpoint.
 * Returns parsed MapData decoded from MessagePack.
 */
export async function loadMap(bbox: BBox): Promise<MapData> {
  const url = `/api/map?south=${bbox.south}&west=${bbox.west}&north=${bbox.north}&east=${bbox.east}`;

  const response = await fetch(url);
  if (!response.ok) {
    const error = await response
      .json()
      .catch(() => ({ error: 'Unknown error' }));
    throw new Error(
      `Failed to load map: ${response.status} - ${error.error || 'Unknown error'}`
    );
  }

  const buffer = await response.arrayBuffer();
  const data = unpack(new Uint8Array(buffer)) as MapData;
  return data;
}

/**
 * Fetch track data from the server's /api/track endpoint.
 * Returns parsed TrackData decoded from MessagePack, or null if no track found.
 */
export async function loadTrack(bbox: BBox): Promise<TrackData | null> {
  const url = `/api/track?south=${bbox.south}&west=${bbox.west}&north=${bbox.north}&east=${bbox.east}`;

  const response = await fetch(url);
  if (response.status === 404) {
    return null;
  }
  if (!response.ok) {
    const error = await response
      .json()
      .catch(() => ({ error: 'Unknown error' }));
    throw new Error(
      `Failed to load track: ${response.status} - ${error.error || 'Unknown error'}`
    );
  }

  const buffer = await response.arrayBuffer();
  const data = unpack(new Uint8Array(buffer)) as TrackData;
  return data;
}

/**
 * Build the complete Three.js scene from map data.
 * Orchestrates all geometry builders.
 */
export function buildScene(data: MapData): THREE.Group {
  const group = new THREE.Group();
  group.name = 'map';

  const proj = new Projection(data.bbox);

  // Terrain (ground plane) — rendered first, at Y=0
  const terrain = buildTerrain(data.bbox, proj);
  group.add(terrain);

  // Water — at Y=-0.1
  const water = buildWater(data.water, proj);
  group.add(water);

  // Nature (parks, forests, trees) — ground at Y=0.01, trees above
  const nature = buildNature(data.parks, data.forests, proj);
  group.add(nature);

  // Roads — at Y=0.05
  const roads = buildRoads(data.roads, proj);
  group.add(roads);

  // Buildings — extruded upward from Y=0
  const buildings = buildBuildings(data.buildings, proj);
  if (buildings) {
    group.add(buildings);
  }

  return group;
}
