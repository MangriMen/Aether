// eslint-disable-next-line import/named
import { EventCallback, listen } from '@tauri-apps/api/event';

import { LoadingPayload } from '../model';

export const listenLoading = <T = LoadingPayload>(callback: EventCallback<T>) =>
  listen<T>('loading', callback);
