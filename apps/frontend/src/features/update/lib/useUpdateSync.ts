import { createEffect, createMemo, on } from 'solid-js';

import { useCheckUpdate } from '../api/updateQueries';
import { checkIsUpdateAvailable } from '../model/checkIsUpdateAvailable';
import {
  UpdateNotificationStyle,
  updateNotificationStyle,
} from '../model/updateNotificationStyle';
import { showUpdateAvailable } from '../ui/showUpdateAvailable';

export const useUpdateSync = () => {
  const update = useCheckUpdate();

  const isUpdateAvailable = createMemo(() =>
    update.data ? checkIsUpdateAvailable(update.data) : false,
  );

  const isUpdateToastEnabled = createMemo(
    () => updateNotificationStyle() === UpdateNotificationStyle.Toast,
  );

  // Switching the toast on while an update waits shows it too.
  createEffect(
    on(
      [isUpdateAvailable, isUpdateToastEnabled],
      ([isAvailable, isToastEnabled]) => {
        if (isToastEnabled && isAvailable && update.data) {
          showUpdateAvailable(update.data);
        }
      },
    ),
  );
};
