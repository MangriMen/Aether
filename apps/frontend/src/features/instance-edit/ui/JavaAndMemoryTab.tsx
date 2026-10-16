import { splitProps, type Component } from 'solid-js';

import { JavaAndMemorySettingsForm } from '@/features/instance-settings';

import type { InstanceSettingsTabProps } from '../model/instanceSettingsTabs';

import { useJavaAndMemorySettingsHandler } from '../lib/useJavaAndMemorySettingsHandler';

export type JavaAndMemoryTabProps = InstanceSettingsTabProps & {
  class?: string;
};

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'editInstance',
    'defaultSettings',
  ]);

  const { initialValues, defaultValues, onChange } =
    useJavaAndMemorySettingsHandler({
      instance: () => local.instance,
      defaultSettings: () => local.defaultSettings,
      editInstance: () => local.editInstance,
    });

  return (
    <JavaAndMemorySettingsForm
      overridable
      initialValues={initialValues}
      defaultValues={defaultValues}
      onChangePartial={onChange}
      {...others}
    />
  );
};
