import { splitProps, type Component } from 'solid-js';

import { HooksSettingsForm } from '@/features/instance-settings/hooks-settings-form';

import type { InstanceSettingsTabProps } from '../model';

import { useHooksSettingsHandler } from '../lib';

export type HooksTabProps = InstanceSettingsTabProps & { class?: string };

export const HooksTab: Component<HooksTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'editInstance']);

  const { initialValues, onChange } = useHooksSettingsHandler({
    instance: () => local.instance,
    editInstance: () => local.editInstance,
  });

  return (
    <HooksSettingsForm
      initialValues={initialValues}
      onChangePartial={onChange}
      {...others}
    />
  );
};
