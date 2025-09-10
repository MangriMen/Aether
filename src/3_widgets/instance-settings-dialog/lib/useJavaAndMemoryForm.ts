import type { FormStore, PartialValues } from '@modular-forms/solid';
import { createForm, setValues, zodForm } from '@modular-forms/solid';
import type {
  JavaAndMemorySettingsSchemaRequiredInput,
  JavaAndMemorySettingsSchemaRequiredOutput,
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
  typeof createForm<JavaAndMemorySettingsSchemaRequiredOutput>
> => {
  const [form, components] =
    createForm<JavaAndMemorySettingsSchemaRequiredOutput>({
      validate: zodForm(JavaAndMemorySettingsSchemaRequired),
      initialValues: {
        memory: { maximum: 512 },
        extraLaunchArgs: '',
        customEnvVars: '',
      },
    });

  return [form, components];
};

export const useResetJavaAndMemoryFormRequiredValues = (
  form: FormStore<JavaAndMemorySettingsSchemaRequiredInput>,
  settings: Accessor<
    PartialValues<JavaAndMemorySettingsSchemaRequiredInput> | undefined
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

export const useResetJavaAndMemoryFormValues = (
  form: FormStore<JavaAndMemorySettingsSchemaValuesInput>,
  settings: Accessor<
    PartialValues<JavaAndMemorySettingsSchemaValuesInput> | undefined
  >,
) => {
  createEffect(() => {
    const javaPath = settings()?.javaPath;
    setValues(form, {
      javaPath,
    });
  });

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
