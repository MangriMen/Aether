import {
  CombinedTooltip,
  SettingsEntry,
  Switch,
  SwitchControl,
  SwitchThumb,
} from '@/shared/ui';
import { createMemo, type Component } from 'solid-js';
import { useAppSettings, useUpdateAppSettings } from '../../api';
import { useColorMode } from '@kobalte/core';

export type ToggleMicaEntryProps = {
  class?: string;
};

export const ToggleMicaEntry: Component<ToggleMicaEntryProps> = (props) => {
  const { colorMode } = useColorMode();

  const appSettings = useAppSettings();
  const updateSettings = useUpdateAppSettings();

  const handleSetMica = async (enabled: boolean) => {
    updateSettings.mutateAsync({
      mica: enabled ? colorMode() : 'off',
    });
  };

  const isDisabled = createMemo(() => !appSettings.data?.transparent);

  return (
    <SettingsEntry title={'Toggle mica'} {...props}>
      <CombinedTooltip
        label={
          isDisabled()
            ? "Can't turn on mica without window transparency"
            : 'Toggle mica'
        }
        as={Switch}
        disabled={isDisabled()}
        checked={appSettings.data?.mica && appSettings.data.mica !== 'off'}
        onChange={handleSetMica}
      >
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
      </CombinedTooltip>
    </SettingsEntry>
  );
};
