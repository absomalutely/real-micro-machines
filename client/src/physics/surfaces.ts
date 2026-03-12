export type SurfaceType = 'road' | 'grass' | 'water' | 'building';

export interface SurfaceProperties {
  speedMultiplier: number;
  dampingMultiplier: number;
}

export const SURFACE_PROPERTIES: Record<SurfaceType, SurfaceProperties> = {
  road: { speedMultiplier: 1.0, dampingMultiplier: 1.0 },
  grass: { speedMultiplier: 0.6, dampingMultiplier: 3.0 },
  water: { speedMultiplier: 0.0, dampingMultiplier: 1.0 },
  building: { speedMultiplier: 1.0, dampingMultiplier: 1.0 },
};

// Collision group constants for Rapier
// Bit 0: ground/grass
// Bit 1: road
// Bit 2: building
// Bit 3: water (sensor)
// Bit 4: car
export const COLLISION_GROUP_GROUND = 0x0001;
export const COLLISION_GROUP_ROAD = 0x0002;
export const COLLISION_GROUP_BUILDING = 0x0004;
export const COLLISION_GROUP_WATER = 0x0008;
export const COLLISION_GROUP_CAR = 0x0010;
