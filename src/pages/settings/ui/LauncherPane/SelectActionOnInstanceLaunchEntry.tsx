import { useTranslate, type Option } from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SettingsEntry,
} from '@/shared/ui';
import type { Component } from 'solid-js';
import { createMemo, createResource } from 'solid-js';
import type { ActionOnInstanceLaunchType } from '../../model';
import {
  getActionOnInstanceLaunch,
  setActionOnInstanceLaunch,
} from '../../api';

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
  const [{ t }] = useTranslate();

  const [value, { refetch }] = createResource(getActionOnInstanceLaunch);

  const currentOption = createMemo(() =>
    ACTION_ON_INSTANCE_LAUNCH_OPTIONS.find(
      (option) => option.value === value(),
    ),
  );

  const handleChangeActionOnInstanceLaunch = async (
    value: Option<ActionOnInstanceLaunchType> | null,
  ) => {
    if (!value) {
      return;
    }

    await setActionOnInstanceLaunch(value.value);
    await refetch();
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
