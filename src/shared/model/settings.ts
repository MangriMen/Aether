import { createSignal } from 'solid-js';

export const IS_DEBUG_KEY = '_AETHER_DEBUG';
export const [isDebug, setIsDebug] = createSignal(
  JSON.parse(localStorage.getItem(IS_DEBUG_KEY) ?? 'false'),
);
