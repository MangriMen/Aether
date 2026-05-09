import type { Accessor } from 'solid-js';

import { createEffect, createSignal } from 'solid-js';

import { isDeveloperMode, setIsDeveloperMode } from '../model';

export const useDeveloperModeCounter = (): [Accessor<number>, () => void] => {
  const [count, setCount] = createSignal(0);

  const increase = () => {
    setCount(count() + 1);
  };

  createEffect(() => {
    if (count() >= 5) {
      setIsDeveloperMode(!isDeveloperMode());
      setCount(0);
    }
  });

  return [count, increase];
};
