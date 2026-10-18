import { describe, expect, it } from 'vitest';

import { formatProgressBytes } from '../progressBytes';

const fmt = {
  formatSize: (n: number) => `${n}B`,
  formatSpeed: (n: number) => `${n}B/s`,
};

describe('formatProgressBytes', () => {
  it('shows done, total and speed', () => {
    expect(
      formatProgressBytes({ bytesDone: 1, bytesTotal: 2, speed: 3 }, fmt),
    ).toBe('1B / 2B · 3B/s');
  });

  it('omits speed when unknown', () => {
    expect(
      formatProgressBytes({ bytesDone: 1, bytesTotal: 2, speed: null }, fmt),
    ).toBe('1B / 2B');
  });

  it('is empty when the size is unknown', () => {
    expect(
      formatProgressBytes({ bytesDone: 1, bytesTotal: null, speed: 3 }, fmt),
    ).toBe('');
  });
});
