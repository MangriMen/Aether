import type { Meta } from 'storybook-solidjs-vite';

import { createEffect, createSignal } from 'solid-js';

type ArgType = NonNullable<Meta['argTypes']>[string];

export const placementArgType: ArgType = {
  control: 'select',
  options: [
    'top',
    'top-start',
    'top-end',
    'bottom',
    'bottom-start',
    'bottom-end',
    'left',
    'left-start',
    'left-end',
    'right',
    'right-start',
    'right-end',
  ],
};

export const gutterArgType: ArgType = {
  control: { type: 'number', min: 0, max: 32 },
};

export const orientationArgType: ArgType = {
  control: 'select',
  options: ['horizontal', 'vertical'],
};

/** A signal that follows a story arg but can also be changed by interaction. */
export const createArgSignal = <T>(read: () => T, fallback: T) => {
  const [value, setValue] = createSignal<T>(fallback);
  createEffect(() => setValue(() => read()));
  return [value, setValue] as const;
};
