import type { BBox, LatLon } from './types';

const DEG2RAD = Math.PI / 180;
const METERS_PER_DEGREE = 111_320;

/**
 * Projects lat/lon coordinates to local XZ meters using a
 * local tangent plane projection centered on the bbox center.
 *
 * - X axis: east (positive) / west (negative)
 * - Z axis: south (positive) / north (negative) — Three.js convention
 * - Y axis: up (handled by builders, not projection)
 */
export class Projection {
  readonly centerLat: number;
  readonly centerLon: number;
  private readonly cosLat: number;

  constructor(bbox: BBox) {
    this.centerLat = (bbox.south + bbox.north) / 2;
    this.centerLon = (bbox.west + bbox.east) / 2;
    this.cosLat = Math.cos(this.centerLat * DEG2RAD);
  }

  /** Project a single lat/lon to local XZ coordinates (meters). */
  project(point: LatLon): { x: number; z: number } {
    const x = (point.lon - this.centerLon) * this.cosLat * METERS_PER_DEGREE;
    const z = -(point.lat - this.centerLat) * METERS_PER_DEGREE;
    return { x, z };
  }

  /** Project an array of lat/lon points. */
  projectAll(points: LatLon[]): { x: number; z: number }[] {
    return points.map((p) => this.project(p));
  }

  /** Get the approximate width and height of a bbox in meters. */
  bboxSizeMeters(bbox: BBox): { width: number; height: number } {
    const width =
      (bbox.east - bbox.west) *
      Math.cos(((bbox.south + bbox.north) / 2) * DEG2RAD) *
      METERS_PER_DEGREE;
    const height = (bbox.north - bbox.south) * METERS_PER_DEGREE;
    return { width, height };
  }
}
