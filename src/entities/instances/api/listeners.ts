import type { ProcessPayload } from '@/entities/events/@x/instances';
import type { EventCallback } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';

export const listenProcess = <T = ProcessPayload>(callback: EventCallback<T>) =>
  listen<T>('process', callback);
