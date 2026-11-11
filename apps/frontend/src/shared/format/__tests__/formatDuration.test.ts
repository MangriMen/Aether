import { describe, expect, it } from 'vitest';

import { formatDuration } from '../formatDuration';

describe('formatDuration', () => {
  it('uses seconds below a minute', () => {
    expect(formatDuration(15, 'en')).toBe('15 seconds');
    expect(formatDuration(1, 'en')).toBe('1 second');
  });

  it('uses whole minutes without seconds', () => {
    expect(formatDuration(600, 'en')).toBe('10 minutes');
    expect(formatDuration(60, 'en')).toBe('1 minute');
  });

  it('combines minutes and seconds when it is not whole', () => {
    expect(formatDuration(90, 'en')).toBe('1 minute 30 seconds');
  });

  it('declines the units in Russian', () => {
    expect(formatDuration(600, 'ru')).toBe('10 минут');
    expect(formatDuration(61, 'ru')).toBe('1 минута 1 секунда');
    expect(formatDuration(22, 'ru')).toBe('22 секунды');
  });
});
