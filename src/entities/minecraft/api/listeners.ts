// eslint-disable-next-line import/named
import { EventCallback, listen } from '@tauri-apps/api/event';

import { LoadingPayload, MinecraftEventName } from '../model';

export const listenEvent = <T = LoadingPayload>(
  event: MinecraftEventName,
  callback: EventCallback<T>,
) => listen<T>(event, callback);
