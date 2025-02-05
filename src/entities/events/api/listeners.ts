import type { EventCallback } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';
import type { LauncherEvent, LauncherEventPayload } from '../model';

export const listenEvent = <
  E extends string = LauncherEvent,
  P = LauncherEventPayload<E>,
>(
  event: E,
  callback: EventCallback<P>,
) => listen<P>(event, callback);
