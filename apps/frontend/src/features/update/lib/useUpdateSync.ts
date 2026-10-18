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

  createEffect(
    on(isUpdateAvailable, (isAvailable) => {
      if (isUpdateToastEnabled() && isAvailable && update.data) {
        showUpdateAvailable(update.data);
      }
    }),
  );
};
