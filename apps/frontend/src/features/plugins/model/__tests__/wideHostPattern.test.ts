import { describe, expect, it } from 'vitest';

import { isWideHostPattern } from '../wideHostPattern';

describe('isWideHostPattern', () => {
  it.each(['*', '*.*', '*.com', '*.*.com', ' * ', 'a*'])(
    'is wide for %j',
    (entry) => {
      expect(isWideHostPattern(entry)).toBe(true);
    },
  );

  it.each(['example.com', '*.example.com', 'api.*.example.com', '203.0.113'])(
    'is not wide for %j',
    (entry) => {
      expect(isWideHostPattern(entry)).toBe(false);
    },
  );
});
