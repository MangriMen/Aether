import { splitProps, type Component } from 'solid-js';

import { HooksSettingsForm } from '@/features/instance-settings/hooks';

import type { InstanceSettingsTabProps } from '../model';

import { useHooksSettingsHandler } from '../lib';

export type HooksTabProps = InstanceSettingsTabProps & { class?: string };

export const HooksTab: Component<HooksTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'editInstance',
    'defaultSettings',
  ]);

  const { initialValues, defaultValues, onChange } = useHooksSettingsHandler({
    instance: () => local.instance,
    editInstance: () => local.editInstance,
    defaultSettings: () => local.defaultSettings,
  });

  return (
    <HooksSettingsForm
      overridable
      initialValues={initialValues}
      defaultValues={defaultValues}
      onChangePartial={onChange}
      {...others}
    />
  );
};
