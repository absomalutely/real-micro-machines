export interface CarConfig {
  maxSpeed: number;
  acceleration: number;
  brakeForce: number;
  turnTorque: number;
  gripFactor: number;
  driftGripFactor: number;
  linearDamping: number;
  angularDamping: number;
  mass: number;
}

export const DEFAULT_CAR_CONFIG: CarConfig = {
  maxSpeed: 30,           // m/s (~108 km/h)
  acceleration: 800,      // force multiplier
  brakeForce: 600,        // braking force
  turnTorque: 15,         // steering torque
  gripFactor: 0.95,       // lateral friction cancellation (high grip)
  driftGripFactor: 0.5,   // reduced grip when drifting
  linearDamping: 2.0,     // natural deceleration
  angularDamping: 5.0,    // prevents endless spinning
  mass: 1500,             // kg
};
