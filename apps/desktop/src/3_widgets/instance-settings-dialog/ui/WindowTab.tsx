import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import { WindowSettingsForm } from '@/features/instance-settings/window-settings-form';

import type { InstanceSettingsTabProps } from '../model';

import { useWindowSettingsHandler } from '../lib';

export type WindowTabProps = InstanceSettingsTabProps & { class?: string };

export const WindowTab: Component<WindowTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'editInstance',
    'defaultSettings',
    'class',
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
