import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import { useEditInstanceIcon } from '@/features/instance';

import { useGeneralSettingsHandler } from '../lib/useGeneralSettingsHandler';
import { type InstanceSettingsTabProps } from '../model/instanceSettingsTabs';
import { GeneralSettingsForm } from './GeneralSettingsForm';

export type GeneralTabProps = InstanceSettingsTabProps & { class?: string };

export const GeneralTab: Component<GeneralTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'editInstance']);

  const editInstanceIcon = useEditInstanceIcon();

  const { initialValues, onChange } = useGeneralSettingsHandler({
    instance: () => local.instance,
    editInstance: () => local.editInstance,
    editInstanceIcon: () => editInstanceIcon.mutateAsync,
  });

  return (
    <GeneralSettingsForm
      realIconSrc={local.instance.iconPath ?? undefined}
      initialValues={initialValues}
      onChangePartial={onChange}
      {...others}
    />
  );
};
