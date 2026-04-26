import { type Component } from 'solid-js';

import { useAppSettings, useEditAppSettings } from '@/entities/settings';
import { useTranslation } from '@/shared/model';
import { SettingsEntry, Switch, SwitchControl, SwitchThumb } from '@/shared/ui';

export type ToggleWindowTransparencyEntryProps = {
  class?: string;
};

export const ToggleWindowTransparencyEntry: Component<
  ToggleWindowTransparencyEntryProps
> = (props) => {
  const [{ t }] = useTranslation();

  const appSettings = useAppSettings();
  const updateSettings = useEditAppSettings();

  const handleSetTransparency = async (enabled: boolean) => {
    await updateSettings.mutateAsync({
      transparent: enabled,
    });
  };

  return (
    <SettingsEntry
      title={t('settings.toggleWindowTransparency')}
      description={t('settings.toggleWindowTransparencyDescription')}
      {...props}
    >
      <Switch
        checked={appSettings.data?.transparent ?? false}
        onChange={handleSetTransparency}
      >
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
      </Switch>
    </SettingsEntry>
  );
};
