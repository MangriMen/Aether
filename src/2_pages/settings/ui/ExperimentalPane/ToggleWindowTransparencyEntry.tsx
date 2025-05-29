import { SettingsEntry, Switch, SwitchControl, SwitchThumb } from '@/shared/ui';
import { type Component } from 'solid-js';
import { useAppSettings, useUpdateAppSettings } from '../../api';

export type ToggleWindowTransparencyEntryProps = {
  class?: string;
};

export const ToggleWindowTransparencyEntry: Component<
  ToggleWindowTransparencyEntryProps
> = (props) => {
  const appSettings = useAppSettings();

  const updateSettings = useUpdateAppSettings();

  const handleSetTransparency = async (enabled: boolean) => {
    await updateSettings.mutateAsync({
      transparent: enabled,
    });
  };

  return (
    <SettingsEntry
      title={'Toggle window transparency'}
      description={'Need to restart app to enable transparency'}
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
