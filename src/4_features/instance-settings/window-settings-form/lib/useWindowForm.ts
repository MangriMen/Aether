import type { FormStore, PartialValues } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect } from 'solid-js';

import type { WindowSchemaValuesInput } from '../model';

import { WindowSchema } from '../model';

export const useWindowForm = (): ReturnType<
  typeof createForm<WindowSchemaValuesInput>
> => {
  const [form, components] = createForm({
    validate: zodForm(WindowSchema),
    initialValues: {
      resolution: undefined,
    },
  });

  return [form, components];
};

export const useResetWindowFormValues = (
  form: FormStore<WindowSchemaValuesInput>,
  initialValues: Accessor<PartialValues<WindowSchemaValuesInput> | undefined>,
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
