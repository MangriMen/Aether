import { createRoot, createSignal } from 'solid-js';

import type { UpdateStatus } from '@/entities/updates';

import { SettingsTab, useSettingsSearchParams } from '@/entities/settings';
import { UpdateAndRestartWarningDialog, useUpdate } from '@/entities/updates';
import { useTranslation } from '@/shared/model';
import { Button, closeToast, showToast } from '@/shared/ui';

let isAlreadyOpened = false;

export const showUpdateAvailable = (update: UpdateStatus) => {
  if (isAlreadyOpened) {
    return;
  }

  isAlreadyOpened = true;

  createRoot((dispose) => {
    const { open } = useSettingsSearchParams();

    const goToUpdatePage = () => {
      closeToast(number);
      open(SettingsTab.Update);
      dispose();
    };

    const [{ t }] = useTranslation();

    const { isUpdating, updateAndRestart } = useUpdate();

    const [isDialogOpen, setIsDialogOpen] = createSignal(false);

    const handleOpenDialog = () => {
      setIsDialogOpen(true);
    };

    const handleInstallUpdate = () => {
      setIsDialogOpen(false);
      updateAndRestart(update);
      closeToast(number);
      dispose();
    };

    const handleCancel = () => {
      setIsDialogOpen(false);
    };

    const number = showToast({
      description: (
        <div class='gap-3 flex flex-col'>
          <span>
            {t('common.version')}
            &nbsp;
            <span class='font-medium'>v{update.version}</span>
            &nbsp;
            {t('settings.updateIsReadyToInstall')}
          </span>
          <div class='gap-4 flex'>
            <Button
              class='h-auto'
              size='sm'
              loading={isUpdating()}
              onClick={handleOpenDialog}
            >
              {t('settings.update')}
            </Button>
            <Button
              class='h-auto'
              size='sm'
              variant='outline'
              onClick={goToUpdatePage}
            >
              {t('settings.goToUpdatePage')}
            </Button>
          </div>
          <UpdateAndRestartWarningDialog
            open={isDialogOpen()}
            onOpenChange={setIsDialogOpen}
            onOk={handleInstallUpdate}
            onCancel={handleCancel}
          />
        </div>
      ),
      persistent: true,
    });
  });
};
