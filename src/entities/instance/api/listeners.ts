// eslint-disable-next-line import/named
import { EventCallback, listen } from '@tauri-apps/api/event';

// eslint-disable-next-line boundaries/element-types
import { ProcessPayload } from '@/entities/minecraft';

export const listenProcess = <T = ProcessPayload>(callback: EventCallback<T>) =>
  listen<T>('process', callback);
