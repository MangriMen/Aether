// eslint-disable-next-line import/named
import type { EventCallback } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';

// eslint-disable-next-line boundaries/element-types
import type { ProcessPayload } from '@/entities/minecraft';

export const listenProcess = <T = ProcessPayload>(callback: EventCallback<T>) =>
  listen<T>('process', callback);
