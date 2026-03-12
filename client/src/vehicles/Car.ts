import * as THREE from 'three';
import type { InputState } from '../core/Input';
import { CarBody } from '../physics/CarBody';
import type { PhysicsWorld } from '../physics/PhysicsWorld';
import { DEFAULT_CAR_CONFIG, type CarConfig } from './CarConfig';

const CAR_COLORS = [
  0xe74c3c, // red
  0x3498db, // blue
  0x2ecc71, // green
  0xf1c40f, // yellow
  0x9b59b6, // purple
  0xe67e22, // orange
  0x1abc9c, // teal
  0xecf0f1, // white
];

export class Car {
  readonly mesh: THREE.Mesh;
  readonly carBody: CarBody;

  private prevPosition = new THREE.Vector3();
  private prevQuaternion = new THREE.Quaternion();
  lastRoadPosition = new THREE.Vector3();

  constructor(world: PhysicsWorld, position: { x: number; y: number; z: number }, heading: number, colorIndex = 0, config: CarConfig = DEFAULT_CAR_CONFIG) {
    // Visual mesh — placeholder box
    const geometry = new THREE.BoxGeometry(2, 1.5, 4); // width, height, length
    const material = new THREE.MeshStandardMaterial({
      color: CAR_COLORS[colorIndex % CAR_COLORS.length],
      roughness: 0.4,
      metalness: 0.6,
    });
    this.mesh = new THREE.Mesh(geometry, material);
    this.mesh.castShadow = true;
    this.mesh.receiveShadow = true;

    // Physics body
    this.carBody = new CarBody(world, config, position, heading);

    // Initialize previous state
    const pos = this.carBody.getPosition();
    this.prevPosition.set(pos.x, pos.y, pos.z);
    this.lastRoadPosition.set(pos.x, pos.y, pos.z);
    const rot = this.carBody.getRotation();
    this.prevQuaternion.set(rot.x, rot.y, rot.z, rot.w);

    // Sync initial visual
    this.syncVisual(1);
  }

  fixedUpdate(input: InputState): void {
    // Store previous state for interpolation
    const pos = this.carBody.getPosition();
    this.prevPosition.set(pos.x, pos.y, pos.z);
    const rot = this.carBody.getRotation();
    this.prevQuaternion.set(rot.x, rot.y, rot.z, rot.w);

    // Apply physics
    this.carBody.applyInput(input);
  }

  /** Interpolate visual between previous and current physics state. */
  syncVisual(alpha: number): void {
    const pos = this.carBody.getPosition();
    const rot = this.carBody.getRotation();

    const currentPos = new THREE.Vector3(pos.x, pos.y, pos.z);
    const currentQuat = new THREE.Quaternion(rot.x, rot.y, rot.z, rot.w);

    this.mesh.position.lerpVectors(this.prevPosition, currentPos, alpha);
    this.mesh.quaternion.slerpQuaternions(this.prevQuaternion, currentQuat, alpha);
  }

  getPosition(): THREE.Vector3 {
    const pos = this.carBody.getPosition();
    return new THREE.Vector3(pos.x, pos.y, pos.z);
  }

  getHeading(): number {
    return this.carBody.getHeading();
  }

  spawn(position: { x: number; y: number; z: number }, heading: number): void {
    this.carBody.teleport(position, heading);
    const pos = this.carBody.getPosition();
    this.prevPosition.set(pos.x, pos.y, pos.z);
    this.lastRoadPosition.set(pos.x, pos.y, pos.z);
    const rot = this.carBody.getRotation();
    this.prevQuaternion.set(rot.x, rot.y, rot.z, rot.w);
    this.syncVisual(1);
  }
}
