import { describe, it, expect, beforeEach } from 'vitest';
import { Input } from './Input';

describe('Input', () => {
  let input: Input;

  beforeEach(() => {
    input = new Input();
  });

  it('returns all zeros with no input', () => {
    const state = input.getState();
    expect(state.throttle).toBe(0);
    expect(state.brake).toBe(0);
    expect(state.steer).toBe(0);
    expect(state.useItem).toBe(false);
  });

  it('W key sets throttle to 1', () => {
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyW' }));
    const state = input.getState();
    expect(state.throttle).toBe(1);
  });

  it('S key sets brake to 1', () => {
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyS' }));
    const state = input.getState();
    expect(state.brake).toBe(1);
  });

  it('A key sets steer to -1', () => {
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyA' }));
    const state = input.getState();
    expect(state.steer).toBe(-1);
  });

  it('D key sets steer to 1', () => {
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyD' }));
    const state = input.getState();
    expect(state.steer).toBe(1);
  });

  it('arrow keys work too', () => {
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'ArrowUp' }));
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'ArrowLeft' }));
    const state = input.getState();
    expect(state.throttle).toBe(1);
    expect(state.steer).toBe(-1);
  });

  it('Space sets useItem to true', () => {
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Space' }));
    const state = input.getState();
    expect(state.useItem).toBe(true);
  });

  it('keyup clears the key', () => {
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyW' }));
    expect(input.getState().throttle).toBe(1);
    window.dispatchEvent(new KeyboardEvent('keyup', { code: 'KeyW' }));
    expect(input.getState().throttle).toBe(0);
  });
});
