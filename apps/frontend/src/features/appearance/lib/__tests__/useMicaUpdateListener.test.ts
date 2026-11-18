import { createRoot } from 'solid-js';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  windowEffect: 'mica_dark' as string | undefined,
  mutate: vi.fn(),
}));

vi.mock('@kobalte/core', () => ({
  useColorMode: () => ({ colorMode: () => 'dark' }),
}));

vi.mock('@/features/settings', () => ({
  useAppSettings: () => ({ data: { windowEffect: mocks.windowEffect } }),
  useEditAppSettings: () => ({ mutate: mocks.mutate }),
}));

vi.mock('../../model/themeContext', () => ({
  useThemeContext: () => [{ rawTheme: 'dark' }],
}));

import { useMicaUpdateListener } from '../useMicaUpdateListener';

let dispose: (() => void) | undefined;

const run = () => {
  createRoot((d) => {
    dispose = d;
    useMicaUpdateListener();
  });
  vi.advanceTimersByTime(100);
};

beforeEach(() => {
  vi.useFakeTimers();
  mocks.mutate.mockClear();
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  vi.useRealTimers();
});

describe('useMicaUpdateListener', () => {
  it('should not mutate settings when the window effect already matches', () => {
    mocks.windowEffect = 'mica_dark';

    run();

    expect(mocks.mutate).not.toHaveBeenCalled();
  });

  it('should mutate settings when the window effect differs from the theme', () => {
    mocks.windowEffect = 'mica_light';

    run();

    expect(mocks.mutate).toHaveBeenCalledWith(
      { windowEffect: 'mica_dark' },
      expect.anything(),
    );
  });
});
