import { SettingsEntry, Switch, SwitchControl, SwitchThumb } from '@/shared/ui';
import { type Component } from 'solid-js';
import { useAppSettings, useUpdateAppSettings } from '../../api';

export type ToggleMicaEntryProps = {
  class?: string;
};

export const ToggleMicaEntry: Component<ToggleMicaEntryProps> = (props) => {
  const appSettings = useAppSettings();

  const updateSettings = useUpdateAppSettings();

  const enableMica = async () => {
    await updateSettings.mutateAsync({
      mica: true,
    });

    setTimeout(() => {
      window.document.documentElement.style = `--transparency: ${0.2}`;
    }, 100);
  };

  const disableMica = async () => {
    window.document.documentElement.style = `--transparency: ${1}`;

    setTimeout(() => {
      updateSettings.mutateAsync({
        mica: false,
      });
    }, 100);
  };

  const handleSetMica = async (enabled: boolean) => {
    if (enabled) {
      await enableMica();
    } else {
      await disableMica();
    }
  };

  return (
    <SettingsEntry title={'Toggle mica'} {...props}>
      <Switch
        checked={appSettings.data?.mica ?? false}
        onChange={handleSetMica}
      >
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
      </Switch>
    </SettingsEntry>
  );
};
