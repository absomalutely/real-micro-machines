import * as THREE from 'three';

const CAMERA_HEIGHT = 250;
const CAMERA_BEHIND = 80;
const CAMERA_LERP = 0.05;
const CAMERA_FOV = 60;
const CAMERA_NEAR = 1;
const CAMERA_FAR = 5000;

export class Camera {
  readonly camera: THREE.PerspectiveCamera;
  private targetPosition = new THREE.Vector3();

  constructor(aspect: number) {
    this.camera = new THREE.PerspectiveCamera(CAMERA_FOV, aspect, CAMERA_NEAR, CAMERA_FAR);
    this.camera.position.set(0, CAMERA_HEIGHT, CAMERA_BEHIND);
    this.camera.lookAt(0, 0, 0);
  }

  setAspect(aspect: number): void {
    this.camera.aspect = aspect;
    this.camera.updateProjectionMatrix();
  }

  /** Snap camera directly to target (no lerp). Use on spawn. */
  snapTo(position: THREE.Vector3, heading: number): void {
    this.targetPosition.copy(position);

    const offset = this.computeOffset(heading);
    this.camera.position.copy(position).add(offset);
    this.camera.lookAt(position.x, 0, position.z);
  }

  /** Smoothly follow the target each frame. */
  follow(position: THREE.Vector3, heading: number): void {
    this.targetPosition.copy(position);

    const offset = this.computeOffset(heading);
    const desiredPos = new THREE.Vector3().copy(position).add(offset);

    this.camera.position.lerp(desiredPos, CAMERA_LERP);
    this.camera.lookAt(position.x, 0, position.z);
  }

  private computeOffset(heading: number): THREE.Vector3 {
    // Heading: 0 = facing -Z (north). Camera sits behind the car.
    const behindX = Math.sin(heading) * CAMERA_BEHIND;
    const behindZ = Math.cos(heading) * CAMERA_BEHIND;
    return new THREE.Vector3(behindX, CAMERA_HEIGHT, behindZ);
  }
}
