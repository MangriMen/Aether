import { SettingsEntry, Switch, SwitchControl, SwitchThumb } from '@/shared/ui';
import type { Component } from 'solid-js';
import { useMica, useSetMica } from '../../api';

export type ToggleMicaEntryProps = {
  class?: string;
};

export const ToggleMicaEntry: Component<ToggleMicaEntryProps> = (props) => {
  const enabled = useMica();

  const setMica = useSetMica();

  const handleSetMica = (enabled: boolean) => {
    window.document.documentElement.style = `--transparency: ${enabled ? 0.2 : 1}`;
    setMica.mutateAsync(enabled);
  };

  return (
    <SettingsEntry title={'Toggle mica'} {...props}>
      <Switch checked={enabled.data ?? false} onChange={handleSetMica}>
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
      </Switch>
    </SettingsEntry>
  );
};
