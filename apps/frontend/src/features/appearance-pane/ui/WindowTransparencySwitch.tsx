import { createSignal, type ComponentProps } from 'solid-js';

import { useAppSettings, useEditAppSettings } from '@/features/settings';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import {
  CombinedDialog,
  Switch,
  SwitchControl,
  SwitchThumb,
} from '@/shared/ui-kit';

import { useRecreateWindow } from '../api/useRecreateWindow';

export const WindowTransparencySwitch = (
  props: Omit<ComponentProps<'div'>, 'onChange'>,
) => {
  const [isOpen, setIsOpen] = createSignal(false);

  const [{ t }] = useTranslation();

  const appSettings = useAppSettings();
  const updateSettings = useEditAppSettings();

  const recreateWindow = useRecreateWindow();

  const handleSetTransparency = async (enabled: boolean) => {
    const isActualTransparent = appSettings.data?.isActualTransparent;

    try {
      await updateSettings.mutateAsync({
        transparent: enabled,
      });
    } catch (error) {
      // The mutation shows the toast; no restart is offered for an unsaved setting.
      logError('Failed to save the transparency setting:', error);
      return;
    }

    if (isActualTransparent !== undefined && isActualTransparent !== enabled) {
      setIsOpen(true);
    }
  };

  const handleOk = (e: MouseEvent) => {
    e.stopPropagation();
    recreateWindow.mutateAsync().catch(logError);
  };

  const handleCancel = (e: MouseEvent) => {
    e.stopPropagation();
    setIsOpen(false);
  };

  return (
    <>
      <Switch
        checked={appSettings.data?.transparent ?? false}
        onChange={handleSetTransparency}
        {...props}
      >
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
      </Switch>
      <CombinedDialog
        open={isOpen()}
        onOpenChange={setIsOpen}
        onOk={handleOk}
        onCancel={handleCancel}
        buttonOkText={t('settings.windowTransparencyDialog.ok')}
        buttonCancelText={t('settings.windowTransparencyDialog.later')}
        header={t('settings.windowTransparencyDialog.header')}
        description={t('settings.windowTransparencyDialog.description')}
      />
    </>
  );
};
