import { makePersisted } from '@solid-primitives/storage';
import { createSignal } from 'solid-js';

export const IS_DEBUG_KEY = '_AETHER_DEBUG';

// eslint-disable-next-line solid/reactivity
export const [isDebug, setIsDebug] = makePersisted(createSignal(false), {
  name: IS_DEBUG_KEY,
});
