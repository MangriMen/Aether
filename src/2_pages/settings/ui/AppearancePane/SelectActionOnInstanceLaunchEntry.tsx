import { useTranslation, type Option } from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SettingsEntry,
} from '@/shared/ui';
import type { Component } from 'solid-js';
import { createMemo } from 'solid-js';
import type { ActionOnInstanceLaunchType } from '../../model';
import { useAppSettings, useUpdateAppSettings } from '../../api';

export type SelectActionOnInstanceLaunchProps = {
  class?: string;
};

type ActionOnInstanceLaunchName = 'doNothing' | 'hide' | 'close';
type ActionOnInstanceLaunchOption = Option<
  ActionOnInstanceLaunchType,
  ActionOnInstanceLaunchName
>;

const ACTION_ON_INSTANCE_LAUNCH_OPTIONS: ActionOnInstanceLaunchOption[] = [
  { value: 'nothing', name: 'doNothing' },
  { value: 'hide', name: 'hide' },
  { value: 'close', name: 'close' },
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
    value: Option<ActionOnInstanceLaunchType> | null,
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
      title={t('settings.actionOnInstanceLaunch')}
      description={t('settings.actionOnInstanceLaunchDescription')}
      {...props}
    >
      <Select
        class='w-44'
        multiple={false}
        options={ACTION_ON_INSTANCE_LAUNCH_OPTIONS}
        optionValue='value'
        optionTextValue='name'
        value={currentOption()}
        onChange={handleChangeActionOnInstanceLaunch}
        itemComponent={(props) => (
          <SelectItem item={props.item}>
            {t(
              `settings.${props.item.textValue as ActionOnInstanceLaunchName}`,
            )}
          </SelectItem>
        )}
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
