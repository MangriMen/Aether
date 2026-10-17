import type { EventCallback, UnlistenFn } from '@tauri-apps/api/event';

import { onCleanup, onMount } from 'solid-js';

type ListenableEvent<T> = {
  listen: (callback: EventCallback<T>) => Promise<UnlistenFn>;
};

// Listens to a backend event while the owner is mounted. `listen` resolves
// asynchronously, so a cleanup that comes first still removes the listener.
export const useEventListener = <T>(
  event: ListenableEvent<T>,
  callback: EventCallback<T>,
) => {
  let unlisten: UnlistenFn | undefined;
  let disposed = false;

  onMount(() => {
    event.listen(callback).then((fn) => {
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
