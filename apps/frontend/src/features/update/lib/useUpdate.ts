import { createMemo } from 'solid-js';

import type { UpdateStatus } from '../model/update';

import { useInstallUpdate } from '../api/updateQueries';
import { checkIsUpdateAvailable } from '../model/checkIsUpdateAvailable';
import { useUpdateStore } from '../model/updateStore';

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
