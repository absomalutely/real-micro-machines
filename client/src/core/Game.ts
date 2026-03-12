import * as THREE from 'three';
import { loadMap, buildScene } from '../map';
import type { BBox } from '../map';
import { Camera } from './Camera';
import { Clock, FIXED_DT } from './Clock';
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

    // Load map
    try {
      if (loadingEl) loadingEl.textContent = 'Fetching map data...';
      const mapData = await loadMap(DEFAULT_BBOX);

      if (loadingEl) loadingEl.textContent = 'Building 3D scene...';
      const mapGroup = buildScene(mapData);
      this.scene.add(mapGroup);

      if (infoEl) {
        infoEl.textContent = `${mapData.roads.length} roads, ${mapData.buildings.length} buildings, ${mapData.parks.length} parks, ${mapData.water.length} water`;
      }
      if (loadingEl) loadingEl.style.display = 'none';

      console.log(`Map loaded: ${mapData.roads.length} roads, ${mapData.buildings.length} buildings`);
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

    const { steps } = this.clock.tick(timestamp);

    // Fixed-timestep updates
    for (let i = 0; i < steps; i++) {
      this.fixedUpdate(FIXED_DT);
    }

    // Render
    this.renderer.render(this.scene, this.cam.camera);
  };

  private fixedUpdate(_dt: number): void {
    // Poll input (will be used by car physics in later steps)
    this.input.getState();
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
