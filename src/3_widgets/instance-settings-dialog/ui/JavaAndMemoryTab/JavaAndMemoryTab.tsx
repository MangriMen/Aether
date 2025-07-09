import type { Component } from 'solid-js';
import { createEffect, createMemo, onCleanup, splitProps } from 'solid-js';

import { cn, debounce } from '@/shared/lib';

import { useEditInstance, type Instance } from '@/entities/instances';

import { useTranslation } from '@/shared/model';

import CustomTextField from './CustomTextField';
import CustomMemory from './CustomMemory';
import { useMaxRam, useSettings } from '@/entities/settings';
import { MIN_JRE_MEMORY, type InstanceSettingsTabProps } from '../../model';
import {
  createForm,
  getError,
  setValue,
  setValues,
  zodForm,
} from '@modular-forms/solid';
import type { JavaAndMemorySettingsSchemaValues } from '../../model/javaAndMemoryValidation';
import { JavaAndMemorySettingsSchema } from '../../model/javaAndMemoryValidation';

export type JavaAndMemoryTabProps = {
  class?: string;
} & InstanceSettingsTabProps;

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const settings = useSettings();
  const maxRam = useMaxRam();

  const maxMemory = createMemo(() =>
    maxRam.data ? maxRam.data / 1024 / 1024 : MIN_JRE_MEMORY,
  );

  const { mutateAsync: editInstance } = useEditInstance();

  const [form, { Form, Field }] = createForm<JavaAndMemorySettingsSchemaValues>(
    {
      validate: zodForm(JavaAndMemorySettingsSchema),
      initialValues: {
        memory: { maximum: null },
      },
    },
  );

  const setMemoryDebounce = debounce(
    async (id: Instance['id'], value: number | null) => {
      editInstance({
        id,
        edit: {
          memory: value ? { maximum: value } : null,
        },
      });
    },
    300,
  );

  const setArguments = async (value: string | null) => {
    const extraLaunchArgs = value !== null ? value?.split(' ') : null;
    await editInstance({
      id: local.instance.id,
      edit: {
        extraLaunchArgs,
      },
    });
  };

  const setEnvironmentVariables = async (value: string | null) => {
    const customEnvVars =
      value !== null
        ? value
            .split(' ')
            .map((variable) => variable.split('=', 2) as [string, string])
        : null;

    console.log(customEnvVars);
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
    setMemoryDebounce.callAndClear();
  });

  createEffect(() => {
    const memory = local.instance.memory?.maximum ?? null;
    setValues(form, {
      memory: {
        maximum: memory,
      },
    });
  });

  const handleChangeMemory = (value: number | null) => {
    setValue(form, 'memory.maximum', value);
    const error = getError(form, 'memory.maximum');

    if (error) {
      return;
    }

    setMemoryDebounce(local.instance.id, value);
  };

  const handleChangeArguments = (value: string | null) => {
    const fieldName = 'extraLaunchArgs';
    setValue(form, fieldName, value);
    const error = getError(form, fieldName);

    if (error) {
      return;
    }

    setArguments(value);
  };

  const handleChangeEnvironmentVariables = (value: string | null) => {
    const fieldName = 'customEnvVars';
    setValue(form, fieldName, value);
    const error = getError(form, fieldName);

    if (error) {
      return;
    }

    setEnvironmentVariables(value);
  };

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Field name='memory.maximum' type='number'>
        {(field) => (
          <CustomMemory
            minValue={MIN_JRE_MEMORY}
            maxValue={maxMemory()}
            defaultValue={settings.data?.memory.maximum}
            value={field.value}
            onChange={handleChangeMemory}
          />
        )}
      </Field>
      <Field name='extraLaunchArgs' type='string'>
        {(field) => (
          <CustomTextField
            fieldLabel={t('instanceSettings.javaArguments')}
            label={t('instanceSettings.customArguments')}
            placeholder={t('instanceSettings.enterArguments')}
            defaultValue={defaultJavaArguments()}
            value={field.value ?? null}
            onChange={handleChangeArguments}
          />
        )}
      </Field>
      <Field name='customEnvVars' type='string'>
        {(field) => (
          <CustomTextField
            fieldLabel={t('instanceSettings.environmentVariables')}
            label={t('instanceSettings.customEnvironmentVariables')}
            placeholder={t('instanceSettings.enterVariables')}
            defaultValue={defaultEnvironmentVariables()}
            value={field.value ?? null}
            onChange={handleChangeEnvironmentVariables}
          />
        )}
      </Field>
    </Form>
  );
};
