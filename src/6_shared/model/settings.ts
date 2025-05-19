import { makePersisted } from '@solid-primitives/storage';
import { createSignal } from 'solid-js';

export const IS_DEBUG_KEY = '_AETHER_DEBUG';

export const [isDebug, setIsDebug] = makePersisted(
  // eslint-disable-next-line solid/reactivity
  createSignal(false),
  {
    name: IS_DEBUG_KEY,
  },
);
