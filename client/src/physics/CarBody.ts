import type { InputState } from '../core/Input';
import type { CarConfig } from '../vehicles/CarConfig';
import { RAPIER, type PhysicsWorld } from './PhysicsWorld';

// Car cuboid half-extents: 4m long, 1.5m tall, 2m wide → half = (1, 0.75, 2)
const HALF_WIDTH = 1;
const HALF_HEIGHT = 0.75;
const HALF_LENGTH = 2;

export class CarBody {
  body: RAPIER.RigidBody;
  private config: CarConfig;

  constructor(world: PhysicsWorld, config: CarConfig, position: { x: number; y: number; z: number }, heading: number) {
    this.config = config;

    const bodyDesc = RAPIER.RigidBodyDesc.dynamic()
      .setTranslation(position.x, position.y + HALF_HEIGHT + 0.1, position.z)
      .setRotation(this.headingToQuat(heading))
      .setLinearDamping(config.linearDamping)
      .setAngularDamping(config.angularDamping);

    this.body = world.createRigidBody(bodyDesc);
    this.body.setAdditionalMass(config.mass, true);

    const colliderDesc = RAPIER.ColliderDesc.cuboid(HALF_WIDTH, HALF_HEIGHT, HALF_LENGTH)
      .setFriction(0.5)
      .setRestitution(0.3);
    world.createCollider(colliderDesc, this.body);
  }

  applyInput(input: InputState): void {
    const rotation = this.body.rotation();
    const forward = this.getForwardDir(rotation);
    const right = this.getRightDir(rotation);

    const vel = this.body.linvel();
    const speed = Math.sqrt(vel.x * vel.x + vel.z * vel.z);
    const isDrifting = input.brake > 0.5 && Math.abs(input.steer) > 0.3;

    // Throttle — forward impulse
    if (input.throttle > 0) {
      const force = this.config.acceleration * input.throttle;
      this.body.applyImpulse(
        { x: forward.x * force, y: 0, z: forward.z * force },
        true,
      );
    }

    // Brake — reverse impulse
    if (input.brake > 0 && !isDrifting) {
      const force = this.config.brakeForce * input.brake;
      this.body.applyImpulse(
        { x: -forward.x * force, y: 0, z: -forward.z * force },
        true,
      );
    }

    // Steering — torque scaled by speed (no turning when stationary)
    if (Math.abs(input.steer) > 0.01) {
      const speedFactor = Math.min(speed / 5, 1);
      const torque = -input.steer * this.config.turnTorque * speedFactor;
      this.body.applyTorqueImpulse({ x: 0, y: torque, z: 0 }, true);
    }

    // Lateral friction cancellation (grip)
    const grip = isDrifting ? this.config.driftGripFactor : this.config.gripFactor;
    const lateralSpeed = vel.x * right.x + vel.z * right.z;
    const cancelForce = -lateralSpeed * grip * this.config.mass;
    this.body.applyImpulse(
      { x: right.x * cancelForce, y: 0, z: right.z * cancelForce },
      true,
    );

    // Speed cap
    if (speed > this.config.maxSpeed) {
      const scale = this.config.maxSpeed / speed;
      this.body.setLinvel({ x: vel.x * scale, y: vel.y, z: vel.z * scale }, true);
    }
  }

  getPosition(): { x: number; y: number; z: number } {
    const t = this.body.translation();
    return { x: t.x, y: t.y, z: t.z };
  }

  getHeading(): number {
    const rot = this.body.rotation();
    // Extract Y-axis rotation from quaternion
    const siny = 2 * (rot.w * rot.y + rot.x * rot.z);
    const cosy = 1 - 2 * (rot.y * rot.y + rot.x * rot.x);
    return Math.atan2(siny, cosy);
  }

  getRotation(): { x: number; y: number; z: number; w: number } {
    const r = this.body.rotation();
    return { x: r.x, y: r.y, z: r.z, w: r.w };
  }

  teleport(position: { x: number; y: number; z: number }, heading: number): void {
    this.body.setTranslation({ x: position.x, y: position.y + HALF_HEIGHT + 0.1, z: position.z }, true);
    this.body.setRotation(this.headingToQuat(heading), true);
    this.body.setLinvel({ x: 0, y: 0, z: 0 }, true);
    this.body.setAngvel({ x: 0, y: 0, z: 0 }, true);
  }

  private getForwardDir(rot: { x: number; y: number; z: number; w: number }): { x: number; z: number } {
    // Car forward is local -Z
    const x = 2 * (rot.x * rot.z + rot.w * rot.y);
    const z = 1 - 2 * (rot.x * rot.x + rot.y * rot.y);
    return { x: -x, z: -z };
  }

  private getRightDir(rot: { x: number; y: number; z: number; w: number }): { x: number; z: number } {
    // Car right is local +X
    const x = 1 - 2 * (rot.y * rot.y + rot.z * rot.z);
    const z = 2 * (rot.x * rot.y - rot.w * rot.z);
    return { x, z };
  }

  private headingToQuat(heading: number): { x: number; y: number; z: number; w: number } {
    // Rotation around Y-axis
    const half = heading / 2;
    return { x: 0, y: Math.sin(half), z: 0, w: Math.cos(half) };
  }
}
