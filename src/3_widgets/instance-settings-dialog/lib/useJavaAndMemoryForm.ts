import type { FormStore } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect } from 'solid-js';

import type { Instance } from '@/entities/instances';

import type { JavaAndMemorySettingsSchemaValues } from '../model/javaAndMemoryValidation';

import { JavaAndMemorySettingsSchema } from '../model/javaAndMemoryValidation';
import { envVarsToString } from './parseEnvVars';
import { extraLaunchArgsToString } from './parseExtraLaunchArgs';

export const useJavaAndMemoryForm = (): ReturnType<
  typeof createForm<JavaAndMemorySettingsSchemaValues>
> => {
  const [form, components] = createForm<JavaAndMemorySettingsSchemaValues>({
    initialValues: {
      customEnvVars: null,
      extraLaunchArgs: null,
      memory: { maximum: null },
    },
    validate: zodForm(JavaAndMemorySettingsSchema),
  });

  return [form, components];
};

export const useResetJavaAndMemoryFormValues = (
  form: FormStore<JavaAndMemorySettingsSchemaValues>,
  instance: Accessor<Instance>,
) => {
  createEffect(() => {
    setValues(form, { memory: { maximum: instance().memory?.maximum } });
  });

  createEffect(() => {
    const extraLaunchArgs = instance().extraLaunchArgs;
    setValues(form, {
      extraLaunchArgs: extraLaunchArgs
        ? extraLaunchArgsToString(extraLaunchArgs)
        : null,
    });
  });

  createEffect(() => {
    const customEnvVars = instance().customEnvVars;
    setValues(form, {
      customEnvVars: customEnvVars ? envVarsToString(customEnvVars) : null,
    });
  });
};
