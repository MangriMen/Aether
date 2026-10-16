import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import { WindowSettingsForm } from '@/features/instance-settings';

import type { InstanceSettingsTabProps } from '../model/instanceSettingsTabs';

import { useWindowSettingsHandler } from '../lib/useWindowSettingsHandler';

export type WindowTabProps = InstanceSettingsTabProps & { class?: string };

export const WindowTab: Component<WindowTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'editInstance',
    'defaultSettings',
  ]);

  const { initialValues, defaultValues, onChange } = useWindowSettingsHandler({
    instance: () => local.instance,
    defaultSettings: () => local.defaultSettings,
    editInstance: () => local.editInstance,
  });

  return (
    <WindowSettingsForm
      overridable
      initialValues={initialValues}
      defaultValues={defaultValues}
      onChangePartial={onChange}
      {...others}
    />
  );
};
