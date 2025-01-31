import type { EventCallback } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';
import type { LoadingPayload, MinecraftEventName } from '../model';

export const listenEvent = <T = LoadingPayload>(
  event: MinecraftEventName,
  callback: EventCallback<T>,
) => listen<T>(event, callback);
