import type { Accessor } from 'solid-js';

import { createSignal } from 'solid-js';

import { isDeveloperMode, setIsDeveloperMode } from '../model/developerMode';

export const useDeveloperModeCounter = (): [Accessor<number>, () => void] => {
  const [count, setCount] = createSignal(0);

  const increase = () => {
    const next = count() + 1;

    if (next >= 5) {
      setIsDeveloperMode(!isDeveloperMode());
      setCount(0);
      return;
    }

    setCount(next);
  };

  return [count, increase];
};
