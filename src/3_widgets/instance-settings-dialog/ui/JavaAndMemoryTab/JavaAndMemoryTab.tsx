import type { Component, ComponentProps } from 'solid-js';
import { createMemo, createResource, onCleanup, splitProps } from 'solid-js';

import { cn, debounce } from '@/shared/lib';

import { useEditInstance, type Instance } from '@/entities/instances';

import { useTranslation } from '@/shared/model';

import CustomTextField from './CustomTextField';
import CustomMemory from './CustomMemory';
import { useMaxRam, useSettings } from '@/entities/settings';
import type { InstanceSettingsTabProps } from '../../model';
import { MIN_JRE_MEMORY } from '../../model';

export type JavaAndMemoryTabProps = ComponentProps<'div'> &
  InstanceSettingsTabProps;

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const settings = useSettings();

  const maxRam = useMaxRam();

  const [maxMemory] = createResource(() => (maxRam.data ?? 0) / 1024 / 1024, {
    initialValue: MIN_JRE_MEMORY,
  });

  const { mutateAsync: editInstance } = useEditInstance();

  const handleChangeMemoryDebounce = debounce(
    async (id: Instance['id'], value: number | null) => {
      editInstance({
        id,
        edit: {
          memory: value ? { maximum: value } : undefined,
        },
      });
    },
    300,
  );

  const handleChangeMemory = (value: number | null) => {
    handleChangeMemoryDebounce(local.instance.id, value);
  };

  const handleChangeArguments = async (value: string | null) => {
    const extraLaunchArgs = value?.split(' ');
    await editInstance({
      id: local.instance.id,
      edit: {
        extraLaunchArgs,
      },
    });
  };

  const handleChangeEnvironmentVariables = async (value: string | null) => {
    const customEnvVars = value
      ?.split(' ')
      .map((variable) => variable.split('=', 2) as [string, string]);
    await editInstance({
      id: local.instance.id,
      edit: {
        customEnvVars,
      },
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
        defaultMaxMemory={settings.data?.memory.maximum}
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
