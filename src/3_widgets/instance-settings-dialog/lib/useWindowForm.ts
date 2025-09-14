import type { FormStore } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect } from 'solid-js';

import type { Instance } from '@/entities/instances';

import type { WindowSchemaValues } from '../model';

import { WindowSchema } from '../model';

export const useWindowForm = (): ReturnType<
  typeof createForm<WindowSchemaValues>
> => {
  const [form, components] = createForm({
    initialValues: {
      resolution: undefined,
    },
    validate: zodForm(WindowSchema),
  });

  return [form, components];
};

export const useResetWindowFormValues = (
  form: FormStore<WindowSchemaValues>,
  instance: Accessor<Instance>,
) => {
  createEffect(() => {
    const resolution = instance().gameResolution;
    if (resolution) {
      setValues(form, {
        resolution: {
          height: resolution[1].toString(),
          width: resolution[0].toString(),
        },
      });
    }
  });
};
