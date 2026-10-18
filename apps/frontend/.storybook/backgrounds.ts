import type { Globals } from 'storybook/internal/types';

// The page around the stories. It is independent of the app theme, which only styles the stories themselves
export const BACKGROUNDS = {
  dark: {
    name: 'Dark',
    value: '#1b1c1d',
    mode: 'dark',
    scrollbarThumb: 'rgb(255 255 255 / 0.25)',
  },
  light: {
    name: 'Light',
    value: '#f6f6f6',
    mode: 'light',
    scrollbarThumb: 'rgb(0 0 0 / 0.25)',
  },
} as const;

export type BackgroundKey = keyof typeof BACKGROUNDS;

export const DEFAULT_BACKGROUND: BackgroundKey = 'dark';

export const getBackground = (globals: Globals) => {
  const selected: unknown = globals['backgrounds'];
  const key =
    typeof selected === 'object' && selected !== null && 'value' in selected
      ? selected.value
      : selected;

  return key === 'light' || key === 'dark'
    ? BACKGROUNDS[key]
    : BACKGROUNDS[DEFAULT_BACKGROUND];
};
