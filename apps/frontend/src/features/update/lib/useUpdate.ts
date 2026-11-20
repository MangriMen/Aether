import { createMemo } from 'solid-js';

import type { UpdateStatus } from '../model/update';

import { useInstallUpdate } from '../api/updateQueries';
import { checkIsUpdateAvailable } from '../model/checkIsUpdateAvailable';
import { useUpdateStore } from '../model/updateStore';

export const useUpdate = () => {
  const [store, setStore] = useUpdateStore();

  const { mutateAsync: installUpdate } = useInstallUpdate();

  /**
   * Whether the update was installed; a failure is shown by the mutation itself. With
   * `allowRunningGames` the games that run are left running across the restart.
   */
  const updateAndRestart = async (
    update: UpdateStatus | null,
    allowRunningGames = false,
  ): Promise<boolean> => {
    if (!checkIsUpdateAvailable(update)) {
      return false;
    }

    setStore('isUpdating', true);

    try {
      await installUpdate(allowRunningGames);
      return true;
    } catch {
      return false;
    } finally {
      setStore('isUpdating', false);
    }
  };

  const isUpdating = createMemo(() => store.isUpdating);

  return { isUpdating, updateAndRestart };
};
