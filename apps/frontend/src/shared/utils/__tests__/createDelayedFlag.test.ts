import { createRoot, createSignal } from 'solid-js';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createDelayedFlag } from '../createDelayedFlag';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

// Effects start once the root has returned, so the timers are driven from outside it.
const setup = () =>
  createRoot((dispose) => {
    const [source, setSource] = createSignal(true);
    const flag = createDelayedFlag(source, 150);

    return { flag, setSource, dispose };
  });

describe('createDelayedFlag', () => {
  it('stays off while the source resolves within the delay', () => {
    const { flag, setSource, dispose } = setup();

    vi.advanceTimersByTime(100);
    setSource(false);
    vi.advanceTimersByTime(500);

    expect(flag()).toBe(false);
    dispose();
  });

  it('turns on after the delay and off as soon as the source does', () => {
    const { flag, setSource, dispose } = setup();

    vi.advanceTimersByTime(150);
    expect(flag()).toBe(true);

    setSource(false);
    expect(flag()).toBe(false);
    dispose();
  });
});
