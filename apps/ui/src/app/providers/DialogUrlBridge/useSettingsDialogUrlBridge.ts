import { useSearchParams } from '@solidjs/router';
import { createEffect } from 'solid-js';

import { closeDialog, showDialog } from '@/shared/model';
import { SettingsDialog } from '@/widgets/settings-view/ui/SettingsDialog';

const modalId = 'global-settings';

export const useSettingsDialogUrlBridge = () => {
  const [searchParams] = useSearchParams();

  createEffect(() => {
    if (searchParams.modal === 'settings') {
      showDialog(modalId, SettingsDialog);
    } else {
      closeDialog(modalId);
    }
  });
};
