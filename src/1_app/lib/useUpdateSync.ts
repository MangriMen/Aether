import { createEffect, createMemo, on } from 'solid-js';

import { useCheckUpdate } from '@/entities/updates';
import { checkIsUpdateAvailable } from '@/entities/updates/model';
import {
  UpdateNotificationStyle,
  updateNotificationStyle,
} from '@/shared/model';

import { showUpdateAvailable } from './showUpdateAvailable';

export const useUpdateSync = () => {
  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  const isUpdateToastEnabled = createMemo(
    () => updateNotificationStyle() === UpdateNotificationStyle.Toast,
  );

  createEffect(
    on(isUpdateAvailable, (isAvailable) => {
      if (isUpdateToastEnabled() && isAvailable && update.data) {
        showUpdateAvailable(update.data);
      }
    }),
  );
};
