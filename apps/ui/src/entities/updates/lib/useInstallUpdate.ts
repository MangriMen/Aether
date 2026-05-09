import { createMemo } from 'solid-js';

import type { UpdateStatus } from '../model';

import { checkIsUpdateAvailable, useInstallUpdate } from '../model';
import { useUpdateStore } from '../model';

export const useUpdate = () => {
  const [store, setStore] = useUpdateStore();

  const { mutateAsync: installUpdate } = useInstallUpdate();

  const updateAndRestart = async (update: UpdateStatus | null) => {
    if (!checkIsUpdateAvailable(update)) {
      return;
    }

    setStore('isUpdating', true);
    try {
      await installUpdate();
    } catch {
      /* empty */
    } finally {
      setStore('isUpdating', false);
    }
  };

  const isUpdating = createMemo(() => store.isUpdating);

  return { isUpdating, updateAndRestart };
};
