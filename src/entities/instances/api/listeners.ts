import type { EventCallback } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';
import type { ProcessPayload } from '../model';

export const listenProcess = <T = ProcessPayload>(callback: EventCallback<T>) =>
  listen<T>('process', callback);
