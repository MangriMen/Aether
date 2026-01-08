import type { Update } from '@tauri-apps/plugin-updater';

import { relaunch } from '@tauri-apps/plugin-process';
import { createMemo } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { showToast } from '@/shared/ui';

import { checkIsUpdateAvailable } from '../model';
import { useUpdateStore } from '../model/updateStore';
import { installUpdate } from './installUpdate';

export const useInstallUpdate = () => {
  const [store, setStore] = useUpdateStore();

  const [{ t }] = useTranslation();

  const updateApp = async (update: Update | null) => {
    if (!checkIsUpdateAvailable(update)) {
      return;
    }

    setStore('isUpdating', true);
    try {
      await installUpdate(update);
    } catch {
      showToast({
        title: t('update.updateError'),
        variant: 'destructive',
      });
    }
    setStore('isUpdating', false);
  };

  const updateAndRestart = async (update: Update | null) => {
    await updateApp(update);
    await relaunch();
  };

  const isUpdating = createMemo(() => store.isUpdating);

  return { isUpdating, updateAndRestart };
};
