import type { FormStore, PartialValues } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect } from 'solid-js';

import {
  JavaAndMemorySettingsSchema,
  type JavaAndMemorySettingsSchemaInput,
} from '../model';

export const useJavaAndMemorySettingsForm = (): ReturnType<
  typeof createForm<JavaAndMemorySettingsSchemaInput>
> => {
  const [form, components] = createForm<JavaAndMemorySettingsSchemaInput>({
    validate: zodForm(JavaAndMemorySettingsSchema),
    initialValues: {
      memory: { maximum: 512 },
      extraLaunchArgs: '',
      customEnvVars: '',
    },
  });

  return [form, components];
};

export const useResetJavaAndMemorySettingsForm = (
  form: FormStore<JavaAndMemorySettingsSchemaInput>,
  settings: Accessor<
    PartialValues<JavaAndMemorySettingsSchemaInput> | undefined
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
