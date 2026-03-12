export interface InputState {
  throttle: number; // 0-1
  brake: number; // 0-1
  steer: number; // -1 (left) to 1 (right)
  useItem: boolean;
}

export class Input {
  private keys = new Set<string>();

  constructor() {
    window.addEventListener('keydown', (e) => this.keys.add(e.code));
    window.addEventListener('keyup', (e) => this.keys.delete(e.code));
    window.addEventListener('blur', () => this.keys.clear());
  }

  getState(): InputState {
    // Try gamepad first
    const gamepad = this.getGamepad();
    if (gamepad) {
      return this.readGamepad(gamepad);
    }
    return this.readKeyboard();
  }

  private readKeyboard(): InputState {
    const throttle = this.keys.has('KeyW') || this.keys.has('ArrowUp') ? 1 : 0;
    const brake = this.keys.has('KeyS') || this.keys.has('ArrowDown') ? 1 : 0;

    let steer = 0;
    if (this.keys.has('KeyA') || this.keys.has('ArrowLeft')) steer -= 1;
    if (this.keys.has('KeyD') || this.keys.has('ArrowRight')) steer += 1;

    const useItem = this.keys.has('Space');

    return { throttle, brake, steer, useItem };
  }

  private readGamepad(gp: Gamepad): InputState {
    const deadzone = 0.15;
    const rawSteer = gp.axes[0] ?? 0;
    const steer = Math.abs(rawSteer) > deadzone ? rawSteer : 0;

    const throttle = Math.max(0, gp.buttons[7]?.value ?? 0); // right trigger
    const brake = Math.max(0, gp.buttons[6]?.value ?? 0); // left trigger
    const useItem = gp.buttons[0]?.pressed ?? false; // A button

    return { throttle, brake, steer, useItem };
  }

  private getGamepad(): Gamepad | null {
    if (typeof navigator.getGamepads !== 'function') return null;
    const gamepads = navigator.getGamepads();
    for (const gp of gamepads) {
      if (gp?.connected) return gp;
    }
    return null;
  }
}
