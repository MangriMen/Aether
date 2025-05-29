import { useThemeContext } from '@/shared/model';
import { SettingsEntry, Switch, SwitchControl, SwitchThumb } from '@/shared/ui';
import { type Component } from 'solid-js';

export type ToggleThemeTransparencyEntryProps = {
  class?: string;
};

export const ToggleThemeTransparencyEntry: Component<
  ToggleThemeTransparencyEntryProps
> = (props) => {
  const [theme, { setTransparency }] = useThemeContext();

  return (
    <SettingsEntry title={'Toggle theme transparency'} {...props}>
      <Switch checked={theme.transparency} onChange={setTransparency}>
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
      </Switch>
    </SettingsEntry>
  );
};
