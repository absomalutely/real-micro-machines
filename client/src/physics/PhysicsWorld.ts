import RAPIER from '@dimforge/rapier3d-compat';
import { FIXED_DT } from '../core/Clock';

export { RAPIER };

export class PhysicsWorld {
  world!: RAPIER.World;
  private initialized = false;

  async init(): Promise<void> {
    await RAPIER.init();
    this.world = new RAPIER.World({ x: 0, y: -9.81, z: 0 });
    this.initialized = true;

    // Ground plane — large static cuboid at Y=-0.5 (top face at Y=0)
    const groundDesc = RAPIER.RigidBodyDesc.fixed().setTranslation(0, -0.5, 0);
    const groundBody = this.world.createRigidBody(groundDesc);
    const groundCollider = RAPIER.ColliderDesc.cuboid(5000, 0.5, 5000);
    this.world.createCollider(groundCollider, groundBody);
  }

  step(): void {
    if (!this.initialized) return;
    this.world.timestep = FIXED_DT;
    this.world.step();
  }

  createRigidBody(desc: RAPIER.RigidBodyDesc): RAPIER.RigidBody {
    return this.world.createRigidBody(desc);
  }

  createCollider(desc: RAPIER.ColliderDesc, body: RAPIER.RigidBody): RAPIER.Collider {
    return this.world.createCollider(desc, body);
  }

  removeRigidBody(body: RAPIER.RigidBody): void {
    this.world.removeRigidBody(body);
  }

  castRay(origin: RAPIER.Vector3, direction: RAPIER.Vector3, maxToi: number): RAPIER.Collider | null {
    const ray = new RAPIER.Ray(origin, direction);
    const hit = this.world.castRay(ray, maxToi, true);
    if (hit) {
      return hit.collider;
    }
    return null;
  }
}
