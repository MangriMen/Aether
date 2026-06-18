import type { Accessor } from 'solid-js';

import { createMediaQuery } from '@solid-primitives/media';

export const createPrefersReducedMotion = (): Accessor<boolean> =>
  createMediaQuery('(prefers-reduced-motion: reduce)');
