export const FIXED_DT = 1 / 60;
const MAX_DT = 0.1;

export interface TickResult {
  steps: number;
  alpha: number;
}

export class Clock {
  private accumulator = 0;
  private previousTime = -1;

  tick(timestamp: number): TickResult {
    if (this.previousTime < 0) {
      this.previousTime = timestamp;
      return { steps: 0, alpha: 0 };
    }

    let dt = (timestamp - this.previousTime) / 1000;
    this.previousTime = timestamp;

    // Cap to prevent spiral of death
    if (dt > MAX_DT) dt = MAX_DT;

    this.accumulator += dt;

    let steps = 0;
    while (this.accumulator >= FIXED_DT) {
      this.accumulator -= FIXED_DT;
      steps++;
    }

    const alpha = this.accumulator / FIXED_DT;
    return { steps, alpha };
  }

  reset(): void {
    this.accumulator = 0;
    this.previousTime = -1;
  }
}
