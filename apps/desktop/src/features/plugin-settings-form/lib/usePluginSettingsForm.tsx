import type { FormStore } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect, on, untrack } from 'solid-js';

import type { PluginSettingsSchemaInput } from '../model';

import { PluginSettingsSchema } from '../model';

export const usePluginSettingsForm = (): ReturnType<
  typeof createForm<PluginSettingsSchemaInput>
> => {
  const [form, components] = createForm<PluginSettingsSchemaInput>({
    validate: zodForm(PluginSettingsSchema),
    initialValues: {
      allowedHosts: [],
      allowedPaths: [],
    },
  });

  return [form, components];
};

export const useResetPluginSettingsFormValues = (
  form: FormStore<PluginSettingsSchemaInput>,
  initialValues: Accessor<PluginSettingsSchemaInput | undefined>,
) => {
  createEffect(
    on([() => form, initialValues], ([formInstance, initial]) => {
      if (initial) {
        untrack(() => {
          setValues(formInstance, initial);
        });
      }
    }),
  );
};
