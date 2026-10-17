import { For, type Component, type ComponentProps, splitProps } from 'solid-js';

import type { FeatureFlagsStore } from '@/features/settings';
import type { SettingsEntryProps } from '@/shared/settings-layout';

import { featureFlagsActions, useFeatureFlagsStore } from '@/features/settings';
import { SettingsEntry } from '@/shared/settings-layout';
import { Switch, SwitchControl, SwitchThumb } from '@/shared/ui-kit';

export type FeatureFlagsEntryProps = ComponentProps<'div'> & SettingsEntryProps;

export const FeatureFlagsEntry: Component<FeatureFlagsEntryProps> = (props) => {
  return (
    <For each={featureFlagsActions.getBooleanFeatureFlags()}>
      {(name) => (
        <SettingsEntry class='capitalize' title={name} {...props}>
          <BooleanFlagSwitch name={name} />
        </SettingsEntry>
      )}
    </For>
  );
};

type BooleanFlagSwitchProps = {
  name: keyof FeatureFlagsStore;
};

const BooleanFlagSwitch: Component<BooleanFlagSwitchProps> = (props) => {
  const [local, others] = splitProps(props, ['name']);

  const [ff, setFf] = useFeatureFlagsStore();

  const value = () => ff[local.name] ?? false;

  const handleChange = (isChecked: boolean) => {
    setFf(local.name, isChecked);
  };

  return (
    <Switch checked={value()} onChange={handleChange} {...others}>
      <SwitchControl>
        <SwitchThumb />
      </SwitchControl>
    </Switch>
  );
};
