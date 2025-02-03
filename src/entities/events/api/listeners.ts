import type { EventCallback } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';
import type { MinecraftEventName, MinecraftEventPayload } from '../model';

export const listenEvent = <T = MinecraftEventPayload>(
  event: MinecraftEventName,
  callback: EventCallback<T>,
) => listen<T>(event, callback);
