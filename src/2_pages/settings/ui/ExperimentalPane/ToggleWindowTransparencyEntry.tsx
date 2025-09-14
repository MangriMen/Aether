import { type Component } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { SettingsEntry, Switch, SwitchControl, SwitchThumb } from '@/shared/ui';

import { useAppSettings, useUpdateAppSettings } from '../../api';

export type ToggleWindowTransparencyEntryProps = {
  class?: string;
};

export const ToggleWindowTransparencyEntry: Component<
  ToggleWindowTransparencyEntryProps
> = (props) => {
  const [{ t }] = useTranslation();

  const appSettings = useAppSettings();
  const updateSettings = useUpdateAppSettings();

  const handleSetTransparency = async (enabled: boolean) => {
    await updateSettings.mutateAsync({
      transparent: enabled,
    });
  };

  return (
    <SettingsEntry
      description={t('settings.toggleWindowTransparencyDescription')}
      title={t('settings.toggleWindowTransparency')}
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
