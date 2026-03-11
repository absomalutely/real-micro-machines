import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { loadMap, buildScene } from './map';
import type { BBox } from './map';

// Default bbox: central London (near Trafalgar Square area)
const DEFAULT_BBOX: BBox = {
  south: 51.505,
  west: -0.130,
  north: 51.515,
  east: -0.115,
};

const app = document.getElementById('app')!;
const loadingEl = document.getElementById('loading')!;
const infoEl = document.getElementById('info')!;

async function init() {
  // Setup renderer
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  app.appendChild(renderer.domElement);

  // Setup scene
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x87ceeb); // Sky blue

  // Setup camera (top-down perspective, looking at center)
  const camera = new THREE.PerspectiveCamera(
    60,
    window.innerWidth / window.innerHeight,
    1,
    5000
  );
  camera.position.set(0, 400, 300);
  camera.lookAt(0, 0, 0);

  // OrbitControls for camera inspection
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.1;
  controls.maxPolarAngle = Math.PI / 2.1; // Don't go below ground
  controls.minDistance = 50;
  controls.maxDistance = 2000;
  controls.target.set(0, 0, 0);

  // Lighting
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
  scene.add(ambientLight);

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
  scene.add(dirLight);

  // Hemisphere light for sky/ground color blending
  const hemiLight = new THREE.HemisphereLight(0x87ceeb, 0x4a7c3f, 0.3);
  scene.add(hemiLight);

  // Handle resize
  window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
  });

  // Render loop
  function animate() {
    requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  }
  animate();

  // Load map data
  try {
    loadingEl.textContent = 'Fetching map data...';
    const mapData = await loadMap(DEFAULT_BBOX);

    loadingEl.textContent = 'Building 3D scene...';
    const mapGroup = buildScene(mapData);
    scene.add(mapGroup);

    // Update info
    infoEl.textContent = `${mapData.roads.length} roads, ${mapData.buildings.length} buildings, ${mapData.parks.length} parks, ${mapData.water.length} water`;

    // Hide loading
    loadingEl.style.display = 'none';

    console.log(
      `Map loaded: ${mapData.roads.length} roads, ${mapData.buildings.length} buildings`
    );
  } catch (error) {
    console.error('Failed to load map:', error);
    loadingEl.textContent = `Failed to load map: ${error instanceof Error ? error.message : 'Unknown error'}`;
    loadingEl.style.color = '#ff6666';
  }
}

init();
