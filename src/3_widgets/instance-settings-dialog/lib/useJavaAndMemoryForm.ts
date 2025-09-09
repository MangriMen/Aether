import type { FormStore, PartialValues } from '@modular-forms/solid';
import { createForm, setValues, zodForm } from '@modular-forms/solid';
import type {
  JavaAndMemorySettingsSchemaRequiredValues,
  JavaAndMemorySettingsSchemaValuesInput,
} from '../model/javaAndMemoryValidation';
import {
  JavaAndMemorySettingsSchema,
  JavaAndMemorySettingsSchemaRequired,
} from '../model/javaAndMemoryValidation';
import type { Accessor } from 'solid-js';
import { createEffect } from 'solid-js';

export const useJavaAndMemoryForm = (): ReturnType<
  typeof createForm<JavaAndMemorySettingsSchemaValuesInput>
> => {
  const [form, components] = createForm<JavaAndMemorySettingsSchemaValuesInput>(
    {
      validate: zodForm(JavaAndMemorySettingsSchema),
      initialValues: {
        memory: { maximum: null },
        extraLaunchArgs: null,
        customEnvVars: null,
      },
    },
  );

  return [form, components];
};

export const useJavaAndMemoryFormRequired = (): ReturnType<
  typeof createForm<JavaAndMemorySettingsSchemaRequiredValues>
> => {
  const [form, components] =
    createForm<JavaAndMemorySettingsSchemaRequiredValues>({
      validate: zodForm(JavaAndMemorySettingsSchemaRequired),
      initialValues: {
        memory: { maximum: 512 },
        extraLaunchArgs: '',
        customEnvVars: '',
      },
    });

  return [form, components];
};

export const useResetJavaAndMemoryFormValues = (
  form: FormStore<
    | JavaAndMemorySettingsSchemaValuesInput
    | JavaAndMemorySettingsSchemaRequiredValues
  >,
  settings: Accessor<
    PartialValues<JavaAndMemorySettingsSchemaValuesInput> | undefined
  >,
) => {
  createEffect(() => {
    const maximum = settings()?.memory?.maximum;
    if (maximum) {
      setValues(form, { memory: { maximum } });
    }
  });

  createEffect(() => {
    const extraLaunchArgs = settings()?.extraLaunchArgs;
    setValues(form, {
      extraLaunchArgs,
    });
  });

  createEffect(() => {
    const customEnvVars = settings()?.customEnvVars;
    setValues(form, {
      customEnvVars,
    });
  });
};
