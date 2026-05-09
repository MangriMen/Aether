import type { FormStore, PartialValues } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect } from 'solid-js';

import type { WindowSettingsSchemaInput } from '../model';

import { WindowSettingsSchema } from '../model';

export const useWindowSettingsForm = (): ReturnType<
  typeof createForm<WindowSettingsSchemaInput>
> => {
  const [form, components] = createForm({
    validate: zodForm(WindowSettingsSchema),
    initialValues: {
      resolution: undefined,
    },
  });

  return [form, components];
};

export const useResetWindowSettingsFormValues = (
  form: FormStore<WindowSettingsSchemaInput>,
  initialValues: Accessor<PartialValues<WindowSettingsSchemaInput> | undefined>,
) => {
  createEffect(() => {
    const { width, height } = initialValues()?.resolution ?? {};

    if (width && height) {
      setValues(form, {
        resolution: {
          width,
          height,
        },
      });
    }
  });
};
