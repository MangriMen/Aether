import type { Component, ComponentProps } from 'solid-js';
import { createMemo, createResource, onCleanup, splitProps } from 'solid-js';

import { cn, debounce } from '@/shared/lib';

import type {
  Instance,
  InstanceEditDto,
  InstanceSettingsTabProps,
} from '@/entities/instances';
import { editInstance } from '@/entities/instances';

import { useTranslate } from '@/shared/model';

import CustomTextField from './CustomTextField';
import CustomMemory from './CustomMemory';
import { getMaxRam, useSettings } from '@/entities/settings';

export type JavaAndMemoryTabProps = ComponentProps<'div'> &
  InstanceSettingsTabProps;

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslate();

  const settings = useSettings();

  const [maxMemory] = createResource(
    () => getMaxRam().then((value) => Math.floor(value / 1024 / 1024)),
    { initialValue: 512 },
  );

  const handleChangeMemoryDebounce = debounce(
    async (id: Instance['id'], value: number | null) => {
      const dto: InstanceEditDto = {
        memory: value ? { maximum: value } : undefined,
      };
      editInstance(id, dto);
    },
    300,
  );

  const handleChangeMemory = (value: number | null) => {
    handleChangeMemoryDebounce(local.instance.id, value);
  };

  const handleChangeArguments = async (value: string | null) => {
    const extraLaunchArgs = value?.split(' ');
    await editInstance(local.instance.id, {
      extraLaunchArgs,
    });
  };

  const handleChangeEnvironmentVariables = async (value: string | null) => {
    const customEnvVars = value
      ?.split(' ')
      .map((variable) => variable.split('=', 2) as [string, string]);
    await editInstance(local.instance.id, {
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

  onCleanup(() => {
    handleChangeMemoryDebounce.callAndClear();
  });

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <CustomMemory
        systemMaxMemory={maxMemory()}
        defaultMaxMemory={settings?.memory.maximum}
        instanceMaxMemory={local.instance.memory?.maximum}
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
