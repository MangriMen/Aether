import type { ComponentProps } from 'solid-js';

import { useAppSettings, useEditAppSettings } from '@/entities/settings';
import { Switch, SwitchControl, SwitchThumb } from '@/shared/ui';

export const WindowTransparencySwitch = (
  props: Omit<ComponentProps<'div'>, 'onChange'>,
) => {
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
      {...props}
    >
      <SwitchControl>
        <SwitchThumb />
      </SwitchControl>
    </Switch>
  );
};
