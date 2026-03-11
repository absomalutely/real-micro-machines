/** A geographic coordinate. */
export interface LatLon {
  lat: number;
  lon: number;
}

/** A geographic bounding box. */
export interface BBox {
  south: number;
  west: number;
  north: number;
  east: number;
}

/** A road segment from OSM data. */
export interface Road {
  id: number;
  highway_type: string;
  points: LatLon[];
  width: number;
  name?: string;
  oneway: boolean;
  lanes: number;
}

/** A building from OSM data. */
export interface Building {
  id: number;
  footprint: LatLon[];
  height: number;
}

/** A generic polygon (water, park, forest). */
export interface MapPolygon {
  id: number;
  points: LatLon[];
  polygon_type: string;
}

/** Complete map data for a region. */
export interface MapData {
  roads: Road[];
  buildings: Building[];
  water: MapPolygon[];
  parks: MapPolygon[];
  forests: MapPolygon[];
  bbox: BBox;
}
