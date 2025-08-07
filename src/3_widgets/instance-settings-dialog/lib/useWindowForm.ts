import type { FormStore } from '@modular-forms/solid';
import { createForm, setValues, zodForm } from '@modular-forms/solid';
import type { Instance } from '@/entities/instances';
import type { Accessor } from 'solid-js';
import { createEffect } from 'solid-js';
import type { WindowSchemaValues } from '../model';
import { WindowSchema } from '../model';

export const useWindowForm = (): ReturnType<
  typeof createForm<WindowSchemaValues>
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
  form: FormStore<WindowSchemaValues>,
  instance: Accessor<Instance>,
) => {
  createEffect(() => {
    const resolution = instance().gameResolution;
    if (resolution) {
      setValues(form, {
        resolution: {
          width: resolution[0].toString(),
          height: resolution[1].toString(),
        },
      });
    }
  });
};
