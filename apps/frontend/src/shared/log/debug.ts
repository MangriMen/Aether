import { makePersisted } from '@solid-primitives/storage';
import { createSignal } from 'solid-js';

const IS_DEBUG_KEY = 'aether:debug';

export const [isDebug, setIsDebug] = makePersisted(createSignal(false), {
  name: IS_DEBUG_KEY,
});
