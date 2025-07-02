import { SettingsEntry, Switch, SwitchControl, SwitchThumb } from '@/shared/ui';
import { type Component } from 'solid-js';
import { useAppSettings, useUpdateAppSettings } from '../../api';
import { useTranslation } from '@/shared/model';

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
