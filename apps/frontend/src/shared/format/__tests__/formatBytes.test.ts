import { describe, expect, it } from 'vitest';

import { formatBytes } from '../formatBytes';

describe('formatBytes', () => {
  it('drops fraction digits for bytes by default', () => {
    expect(formatBytes(512, 'en')).toBe('512 byte');
  });

  it('uses a fixed number of fraction digits when asked', () => {
    expect(formatBytes(1536, 'en', { fractionDigits: 2 })).toBe('1.50 kB');
    expect(formatBytes(512, 'en', { fractionDigits: 1 })).toBe('512.0 byte');
  });
});
