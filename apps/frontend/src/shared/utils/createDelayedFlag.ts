import { createEffect, createSignal, onCleanup, type Accessor } from 'solid-js';

/**
 * Turns on `delay` ms after `source` does and off right away, so a state that resolves quickly
 * (a skeleton for a cached query) never flashes on screen.
 */
export const createDelayedFlag = (
  source: Accessor<boolean>,
  delay = 150,
): Accessor<boolean> => {
  const [flag, setFlag] = createSignal(false);

  createEffect(() => {
    if (!source()) {
      setFlag(false);
      return;
    }

    const timeout = setTimeout(() => setFlag(true), delay);
    onCleanup(() => clearTimeout(timeout));
  });

  return flag;
};
