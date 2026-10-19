import type { EventCallback, UnlistenFn } from '@tauri-apps/api/event';

import { onCleanup, onMount } from 'solid-js';

import { logError } from '@/shared/log';

type ListenableEvent<T> = {
  listen: (callback: EventCallback<T>) => Promise<UnlistenFn>;
};

// Listens to a backend event while the owner is mounted. `listen` resolves
// asynchronously, so a cleanup that comes first still removes the listener.
//
// The backend numbers its events in the order it sends them. The webview can hand over an
// event it has delivered already, long after, so one that does not follow the last this
// listener saw is dropped.
export const useEventListener = <T extends { seq: number }>(
  event: ListenableEvent<T>,
  callback: EventCallback<T>,
) => {
  let unlisten: UnlistenFn | undefined;
  let disposed = false;
  let lastSeq = 0;

  const onEvent: EventCallback<T> = (e) => {
    if (e.payload.seq <= lastSeq) {
      logError('[EVENT] dropped an event heard already', e.payload);
      return;
    }

    lastSeq = e.payload.seq;
    callback(e);
  };

  onMount(() => {
    event.listen(onEvent).then((fn) => {
      if (disposed) {
        fn();
      } else {
        unlisten = fn;
      }
    });
  });

  onCleanup(() => {
    disposed = true;
    unlisten?.();
  });
};
