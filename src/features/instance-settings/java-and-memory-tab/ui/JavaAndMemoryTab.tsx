import { debounce } from '@solid-primitives/scheduled';
import { Component, ComponentProps, createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import {
  editMinecraftInstance,
  Instance,
  InstanceEditDto,
  InstanceSettingsTabProps,
} from '@/entities/instance';

import CustomMemory from './CustomMemory';
import CustomTextField from './CustomTextField';

export type JavaAndMemoryTabProps = ComponentProps<'div'> &
  InstanceSettingsTabProps;

const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const handleChangeMemoryDebounce = debounce(
    (id: Instance['id'], value: number | null) => {
      const dto: InstanceEditDto = {
        memory: value ? { maximum: value } : undefined,
      };
      editMinecraftInstance(id, dto);
    },
    1000,
  );

  const handleChangeMemory = (value: number | null) => {
    handleChangeMemoryDebounce(local.instance.id, value);
  };

  const handleChangeArguments = (value: string | null) => {
    const extraLaunchArgs = value?.split(' ');
    editMinecraftInstance(local.instance.id, {
      extraLaunchArgs,
    });
  };

  const handleChangeEnvironmentVariables = (value: string | null) => {
    const customEnvVars = value
      ?.split(' ')
      .map((variable) => variable.split('=', 2) as [string, string]);
    editMinecraftInstance(local.instance.id, {
      customEnvVars,
    });
  };

  const defaultJavaArguments = createMemo(() =>
    local.instance.extraLaunchArgs?.join(' '),
  );
  const defaultEnvironmentVariables = createMemo(() =>
    local.instance.customEnvVars
      ?.map(([key, value]) => `${key}=${value}`)
      ?.join(' '),
  );

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <CustomMemory
        defaultValue={local.instance.memory?.maximum}
        onChange={handleChangeMemory}
      />
      <CustomTextField
        defaultValue={defaultJavaArguments()}
        fieldLabel='Java arguments'
        label='Custom arguments'
        placeholder='Enter arguments'
        onChange={handleChangeArguments}
      />
      <CustomTextField
        defaultValue={defaultEnvironmentVariables()}
        fieldLabel='Environment arguments'
        label='Custom environment variables'
        placeholder='Enter environment variables'
        onChange={handleChangeEnvironmentVariables}
      />
    </div>
  );
};

export default JavaAndMemoryTab;
