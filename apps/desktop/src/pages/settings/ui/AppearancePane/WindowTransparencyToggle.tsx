import { useAppSettings, useEditAppSettings } from '@/entities/settings';
import { Switch, SwitchControl, SwitchThumb } from '@/shared/ui';

export const WindowTransparencySwitch = () => {
  const appSettings = useAppSettings();
  const updateSettings = useEditAppSettings();

  const handleSetTransparency = async (enabled: boolean) => {
    await updateSettings.mutateAsync({
      transparent: enabled,
    });
  };

  return (
    <Switch
      checked={appSettings.data?.transparent ?? false}
      onChange={handleSetTransparency}
    >
      <SwitchControl>
        <SwitchThumb />
      </SwitchControl>
    </Switch>
  );
};
