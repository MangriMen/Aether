import { debounce } from '@solid-primitives/scheduled';
import type { Component, ComponentProps } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import type {
  Instance,
  InstanceEditDto,
  InstanceSettingsTabProps,
} from '@/entities/instances';
import { editMinecraftInstance } from '@/entities/instances';

import { useTranslate } from '@/shared/model';

import CustomMemory from './CustomMemory';
import CustomTextField from './CustomTextField';

export type JavaAndMemoryProps = ComponentProps<'div'> &
  InstanceSettingsTabProps;

export const JavaAndMemory: Component<JavaAndMemoryProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslate();

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
        fieldLabel={t('instanceSettings.javaArguments')}
        label={t('instanceSettings.customArguments')}
        placeholder={t('instanceSettings.enterArguments')}
        onChange={handleChangeArguments}
      />
      <CustomTextField
        defaultValue={defaultEnvironmentVariables()}
        fieldLabel={t('instanceSettings.environmentVariables')}
        label={t('instanceSettings.customEnvironmentVariables')}
        placeholder={t('instanceSettings.enterVariables')}
        onChange={handleChangeEnvironmentVariables}
      />
    </div>
  );
};
