import { makePersisted } from '@solid-primitives/storage';
import { createSignal } from 'solid-js';

export const IS_DEVELOPER_MODE_KEY = 'aether:developer-mode';

export const [isDeveloperMode, setIsDeveloperMode] = makePersisted(
  createSignal(false),
  { name: IS_DEVELOPER_MODE_KEY },
);
