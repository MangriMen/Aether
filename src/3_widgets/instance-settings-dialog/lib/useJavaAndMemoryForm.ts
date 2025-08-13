import type { FormStore } from '@modular-forms/solid';
import { createForm, setValues, zodForm } from '@modular-forms/solid';
import type { JavaAndMemorySettingsSchemaValues } from '../model/javaAndMemoryValidation';
import { JavaAndMemorySettingsSchema } from '../model/javaAndMemoryValidation';
import type { Instance } from '@/entities/instances';
import type { Accessor } from 'solid-js';
import { createEffect } from 'solid-js';
import { extraLaunchArgsToString } from './parseExtraLaunchArgs';
import { envVarsToString } from './parseEnvVars';

export const useJavaAndMemoryForm = (): ReturnType<
  typeof createForm<JavaAndMemorySettingsSchemaValues>
> => {
  const [form, components] = createForm<JavaAndMemorySettingsSchemaValues>({
    validate: zodForm(JavaAndMemorySettingsSchema),
    initialValues: {
      memory: { maximum: null },
      extraLaunchArgs: null,
      customEnvVars: null,
    },
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
