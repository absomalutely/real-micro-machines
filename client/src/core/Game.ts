import * as THREE from 'three';
import { loadMap, loadTrack, buildScene, Projection } from '../map';
import type { BBox, TrackData } from '../map';
import { PhysicsWorld } from '../physics/PhysicsWorld';
import { buildTrackColliders } from '../physics/TrackColliders';
import { Car } from '../vehicles/Car';
import { Camera } from './Camera';
import { Clock } from './Clock';
import { Input } from './Input';
import { Renderer } from './Renderer';

const DEFAULT_BBOX: BBox = {
  south: 50.966,
  west: 0.243,
  north: 50.978,
  east: 0.263,
};

export class Game {
  private renderer!: Renderer;
  private scene!: THREE.Scene;
  private cam!: Camera;
  private input!: Input;
  private clock!: Clock;
  private physics!: PhysicsWorld;
  private car: Car | null = null;
  private running = false;

  async init(): Promise<void> {
    const app = document.getElementById('app')!;
    const loadingEl = document.getElementById('loading');
    const infoEl = document.getElementById('info');

    // Core systems
    this.renderer = new Renderer(app);
    this.input = new Input();
    this.clock = new Clock();

    // Scene
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x87ceeb);

    // Camera
    this.cam = new Camera(window.innerWidth / window.innerHeight);

    window.addEventListener('resize', () => {
      this.cam.setAspect(window.innerWidth / window.innerHeight);
    });

    // Lighting
    this.setupLighting();

    // Physics
    if (loadingEl) loadingEl.textContent = 'Initializing physics...';
    this.physics = new PhysicsWorld();
    await this.physics.init();

    // Load map + track
    try {
      if (loadingEl) loadingEl.textContent = 'Fetching map data...';
      const mapData = await loadMap(DEFAULT_BBOX);

      if (loadingEl) loadingEl.textContent = 'Building 3D scene...';
      const mapGroup = buildScene(mapData);
      this.scene.add(mapGroup);

      if (infoEl) {
        infoEl.textContent = `${mapData.roads.length} roads, ${mapData.buildings.length} buildings, ${mapData.parks.length} parks, ${mapData.water.length} water`;
      }

      console.log(`Map loaded: ${mapData.roads.length} roads, ${mapData.buildings.length} buildings`);

      // Build physics colliders from map geometry
      if (loadingEl) loadingEl.textContent = 'Building physics...';
      const projection = new Projection(mapData.bbox);
      buildTrackColliders(mapData, projection, this.physics);

      // Fetch track data for start line position
      if (loadingEl) loadingEl.textContent = 'Loading track...';
      let trackData: TrackData | null = null;
      try {
        trackData = await loadTrack(DEFAULT_BBOX);
      } catch (e) {
        console.warn('Track generation failed, spawning at origin:', e);
      }

      // Spawn car
      const spawnPos = this.getSpawnPosition(trackData, projection);
      const spawnHeading = this.getSpawnHeading(trackData);

      this.car = new Car(this.physics, spawnPos, spawnHeading);
      this.scene.add(this.car.mesh);

      // Snap camera to car
      this.cam.snapTo(this.car.getPosition(), this.car.getHeading());

      if (loadingEl) loadingEl.style.display = 'none';

      console.log(`Car spawned at (${spawnPos.x.toFixed(1)}, ${spawnPos.y.toFixed(1)}, ${spawnPos.z.toFixed(1)})`);
    } catch (error) {
      console.error('Failed to load map:', error);
      if (loadingEl) {
        loadingEl.textContent = `Failed to load map: ${error instanceof Error ? error.message : 'Unknown error'}`;
        loadingEl.style.color = '#ff6666';
      }
    }
  }

  start(): void {
    if (this.running) return;
    this.running = true;
    requestAnimationFrame(this.loop);
  }

  private loop = (timestamp: number): void => {
    if (!this.running) return;
    requestAnimationFrame(this.loop);

    const { steps, alpha } = this.clock.tick(timestamp);

    // Fixed-timestep updates
    for (let i = 0; i < steps; i++) {
      this.fixedUpdate();
    }

    // Interpolate and render
    if (this.car) {
      this.car.syncVisual(alpha);
      this.cam.follow(this.car.getPosition(), this.car.getHeading());
    }

    this.renderer.render(this.scene, this.cam.camera);
  };

  private fixedUpdate(): void {
    const input = this.input.getState();

    if (this.car) {
      this.car.fixedUpdate(input, this.physics);
    }

    this.physics.step();
  }

  private getSpawnPosition(trackData: TrackData | null, projection: Projection): { x: number; y: number; z: number } {
    if (trackData?.start_line) {
      const [lat, lon] = trackData.start_line.position;
      const projected = projection.project({ lat, lon });
      return { x: projected.x, y: 0, z: projected.z };
    }
    // Fallback: center of map
    return { x: 0, y: 0, z: 0 };
  }

  private getSpawnHeading(trackData: TrackData | null): number {
    if (trackData?.start_line) {
      return trackData.start_line.heading;
    }
    return 0;
  }

  private setupLighting(): void {
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
    this.scene.add(ambientLight);

    const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
    dirLight.position.set(200, 400, 200);
    dirLight.castShadow = true;
    dirLight.shadow.mapSize.width = 2048;
    dirLight.shadow.mapSize.height = 2048;
    dirLight.shadow.camera.near = 1;
    dirLight.shadow.camera.far = 1500;
    dirLight.shadow.camera.left = -500;
    dirLight.shadow.camera.right = 500;
    dirLight.shadow.camera.top = 500;
    dirLight.shadow.camera.bottom = -500;
    this.scene.add(dirLight);

    const hemiLight = new THREE.HemisphereLight(0x87ceeb, 0x4a7c3f, 0.3);
    this.scene.add(hemiLight);
  }
}
