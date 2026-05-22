import { useSearchParams } from '@solidjs/router';
import { createEffect } from 'solid-js';

import { useSettingsSearchParams } from '@/entities/settings';
import { closeDialog, showDialog } from '@/shared/model';
import { SettingsDialog } from '@/widgets/settings-view/ui/SettingsDialog';

const modalId = 'global-settings';

export const useSettingsDialogUrlBridge = () => {
  const [searchParams] = useSearchParams();
  const { close } = useSettingsSearchParams();

  createEffect(() => {
    if (searchParams.modal === 'settings') {
      showDialog(modalId, SettingsDialog, {
        onOpenChange: (isOpen) => {
          if (!isOpen) {
            close();
          }
        },
      });
    } else {
      closeDialog(modalId);
    }
  });
};
