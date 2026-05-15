import { useSearchParams } from '@solidjs/router';
import { createEffect } from 'solid-js';

import { closeDialog, showDialog } from '@/shared/model';
import { SettingsDialog } from '@/widgets/settings-view/ui/SettingsDialog';

export const DialogUrlBridge = () => {
  const [searchParams, setSearchParams] = useSearchParams();

  createEffect(() => {
    const modalId = 'global-settings';

    if (searchParams.modal === 'settings') {
      showDialog(modalId, SettingsDialog, {
        defaultOpen: true,
        onOpenChange: (open: boolean) => {
          if (!open) {
            setSearchParams({ modal: undefined, tab: undefined });
          }
        },
      });
    } else {
      closeDialog(modalId);
    }
  });

  return null;
};
