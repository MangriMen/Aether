import { createRoot } from 'solid-js';
import { afterEach, describe, expect, it } from 'vitest';

import { isDeveloperMode, setIsDeveloperMode } from '../../model/developerMode';
import { useDeveloperModeCounter } from '../useDeveloperModeCounter';

describe('useDeveloperModeCounter', () => {
  afterEach(() => {
    setIsDeveloperMode(false);
  });

  it('should toggle developer mode and reset the count when it is increased five times', () => {
    createRoot((dispose) => {
      const [count, increase] = useDeveloperModeCounter();

      for (let i = 0; i < 4; i++) {
        increase();
      }

      expect(count()).toBe(4);
      expect(isDeveloperMode()).toBe(false);

      increase();
      expect(isDeveloperMode()).toBe(true);
      expect(count()).toBe(0);

      dispose();
    });
  });
});
