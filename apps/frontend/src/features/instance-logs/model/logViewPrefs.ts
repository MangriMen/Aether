import { makePersisted } from '@solid-primitives/storage';
import { createSignal } from 'solid-js';

const WRAP_LINES_KEY = 'aether:logs-wrap-lines';

/** Whether long log lines continue on the next row instead of scrolling sideways. */
export const [wrapLines, setWrapLines] = makePersisted(createSignal(false), {
  name: WRAP_LINES_KEY,
});
