import type { Component } from 'solid-js';
import { createEffect, createMemo, onCleanup, splitProps } from 'solid-js';

import { cn, debounce } from '@/shared/lib';

import { useEditInstance } from '@/entities/instances';

import { useTranslation } from '@/shared/model';

import CustomTextField from './CustomTextField';
import CustomMemory from './CustomMemory';
import { useMaxRam, useSettings } from '@/entities/settings';
import {
  MEMORY_SLIDER_HANDLE_DEBOUNCE,
  MIN_JRE_MEMORY,
  type InstanceSettingsTabProps,
} from '../../model';
import type { FieldPath } from '@modular-forms/solid';
import {
  createForm,
  getError,
  setValue,
  setValues,
  zodForm,
} from '@modular-forms/solid';
import type { JavaAndMemorySettingsSchemaValues } from '../../model/javaAndMemoryValidation';
import { JavaAndMemorySettingsSchema } from '../../model/javaAndMemoryValidation';
import {
  bytesToMegabytes,
  stringToEnvVars,
  stringToExtraLaunchArgs,
  envVarsToString,
  extraLaunchArgsToString,
} from '../../lib';

type JavaFieldPath = FieldPath<JavaAndMemorySettingsSchemaValues>;

export type JavaAndMemoryTabProps = {
  class?: string;
} & InstanceSettingsTabProps;

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const settings = useSettings();
  const maxRam = useMaxRam();

  const maxMemory = createMemo(() =>
    maxRam.data ? bytesToMegabytes(maxRam.data) : MIN_JRE_MEMORY,
  );

  const defaultExtraLaunchArgs = createMemo(() => {
    if (!local.instance.extraLaunchArgs) {
      return;
    }
    return extraLaunchArgsToString(local.instance.extraLaunchArgs);
  });
  const defaultCustomEnvVars = createMemo(() => {
    if (!local.instance.customEnvVars) {
      return;
    }
    return envVarsToString(local.instance.customEnvVars);
  });

  const [form, { Form, Field }] = createForm<JavaAndMemorySettingsSchemaValues>(
    {
      validate: zodForm(JavaAndMemorySettingsSchema),
      initialValues: {
        memory: { maximum: null },
      },
    },
  );

  createEffect(() => {
    const memory = local.instance.memory?.maximum ?? null;
    setValues(form, {
      memory: {
        maximum: memory,
      },
      extraLaunchArgs: defaultExtraLaunchArgs() ?? null,
      customEnvVars: defaultCustomEnvVars() ?? null,
    });
  });

  const { mutateAsync: editInstance } = useEditInstance();

  const editInstanceDebounced = debounce(
    editInstance,
    MEMORY_SLIDER_HANDLE_DEBOUNCE,
  );
  onCleanup(() => {
    editInstanceDebounced.callAndClear();
  });

  const updateMemory = (value: number | null) => {
    const fieldName: Extract<JavaFieldPath, 'memory.maximum'> =
      'memory.maximum';

    setValue(form, fieldName, value);

    if (!getError(form, fieldName)) {
      editInstanceDebounced({
        id: local.instance.id,
        edit: {
          memory: value ? { maximum: value } : null,
        },
      });
    }
  };

  const updateExtraLaunchArgs = (value: string | null) => {
    const fieldName: Extract<JavaFieldPath, 'extraLaunchArgs'> =
      'extraLaunchArgs';

    setValue(form, fieldName, value);

    if (!getError(form, fieldName)) {
      editInstance({
        id: local.instance.id,
        edit: {
          extraLaunchArgs: value ? stringToExtraLaunchArgs(value) : null,
        },
      });
    }
  };

  const updateEnvVars = (value: string | null) => {
    const fieldName: Extract<JavaFieldPath, 'customEnvVars'> = 'customEnvVars';

    setValue(form, fieldName, value);

    if (!getError(form, fieldName)) {
      editInstance({
        id: local.instance.id,
        edit: {
          customEnvVars: value ? stringToEnvVars(value) : null,
        },
      });
    }
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
            onChange={updateMemory}
          />
        )}
      </Field>
      <Field name='extraLaunchArgs' type='string'>
        {(field) => (
          <CustomTextField
            fieldLabel={t('instanceSettings.javaArguments')}
            label={t('instanceSettings.customArguments')}
            placeholder={t('instanceSettings.enterArguments')}
            defaultValue={defaultExtraLaunchArgs()}
            value={field.value}
            onChange={updateExtraLaunchArgs}
          />
        )}
      </Field>
      <Field name='customEnvVars' type='string'>
        {(field) => (
          <CustomTextField
            fieldLabel={t('instanceSettings.environmentVariables')}
            label={t('instanceSettings.customEnvironmentVariables')}
            placeholder={t('instanceSettings.enterVariables')}
            defaultValue={defaultCustomEnvVars()}
            value={field.value}
            onChange={updateEnvVars}
          />
        )}
      </Field>
    </Form>
  );
};
