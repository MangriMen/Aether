import { createEffect, createMemo, on } from 'solid-js';

import {
  UpdateNotificationStyle,
  updateNotificationStyle,
} from '@/shared/model';

import { useCheckUpdate } from '../api/updateQueries';
import { checkIsUpdateAvailable } from '../model/checkIsUpdateAvailable';
import { showUpdateAvailable } from '../ui/UpdateAvailableToast';

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
