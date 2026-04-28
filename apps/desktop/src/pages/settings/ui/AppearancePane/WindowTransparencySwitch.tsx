import { createSignal, type ComponentProps } from 'solid-js';

import { useRecreateWindow } from '@/entities/application';
import { useAppSettings, useEditAppSettings } from '@/entities/settings';
import { useTranslation } from '@/shared/model';
import {
  CombinedDialog,
  Switch,
  SwitchControl,
  SwitchThumb,
} from '@/shared/ui';

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

    if (isActualTransparent !== undefined && isActualTransparent !== enabled) {
      setIsOpen(true);
    }

    await updateSettings.mutateAsync({
      transparent: enabled,
    });
  };

  const handleOk = (e: MouseEvent) => {
    e.stopPropagation();
    recreateWindow.mutateAsync();
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
