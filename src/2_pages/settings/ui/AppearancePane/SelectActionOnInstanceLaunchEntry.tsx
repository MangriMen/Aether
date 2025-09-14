import type { Component } from 'solid-js';

import { createMemo } from 'solid-js';

import { type Option, useTranslation } from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SettingsEntry,
} from '@/shared/ui';

import type { ActionOnInstanceLaunchType } from '../../model';

import { useAppSettings, useUpdateAppSettings } from '../../api';

export type SelectActionOnInstanceLaunchProps = {
  class?: string;
};

type ActionOnInstanceLaunchName = 'close' | 'doNothing' | 'hide';
type ActionOnInstanceLaunchOption = Option<
  ActionOnInstanceLaunchType,
  ActionOnInstanceLaunchName
>;

const ACTION_ON_INSTANCE_LAUNCH_OPTIONS: ActionOnInstanceLaunchOption[] = [
  { name: 'doNothing', value: 'nothing' },
  { name: 'hide', value: 'hide' },
  { name: 'close', value: 'close' },
];

export const SelectActionOnInstanceLaunchEntry: Component<
  SelectActionOnInstanceLaunchProps
> = (props) => {
  const [{ t }] = useTranslation();

  const appSettings = useAppSettings();

  const currentOption = createMemo(() =>
    ACTION_ON_INSTANCE_LAUNCH_OPTIONS.find(
      (option) => option.value === appSettings.data?.actionOnInstanceLaunch,
    ),
  );

  const updateAppSettings = useUpdateAppSettings();
  const handleChangeActionOnInstanceLaunch = async (
    value: null | Option<ActionOnInstanceLaunchType>,
  ) => {
    if (!value) {
      return;
    }

    await updateAppSettings.mutateAsync({
      actionOnInstanceLaunch: value.value,
    });
  };

  return (
    <SettingsEntry
      description={t('settings.actionOnInstanceLaunchDescription')}
      title={t('settings.actionOnInstanceLaunch')}
      {...props}
    >
      <Select
        class='w-40 min-w-40'
        itemComponent={(props) => (
          <SelectItem item={props.item}>
            {t(
              `settings.${props.item.textValue as ActionOnInstanceLaunchName}`,
            )}
          </SelectItem>
        )}
        multiple={false}
        onChange={handleChangeActionOnInstanceLaunch}
        options={ACTION_ON_INSTANCE_LAUNCH_OPTIONS}
        optionTextValue='name'
        optionValue='value'
        value={currentOption()}
      >
        <SelectTrigger>
          <SelectValue<ActionOnInstanceLaunchOption>>
            {(state) => t(`settings.${state.selectedOption().name}`)}
          </SelectValue>
        </SelectTrigger>
        <SelectContent />
      </Select>
    </SettingsEntry>
  );
};
