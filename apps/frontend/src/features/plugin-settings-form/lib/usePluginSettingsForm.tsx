import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, setInput } from '@formisch/solid';
import { createEffect, on, untrack } from 'solid-js';

import { PluginSettingsSchema, type PluginSettingsSchemaInput } from '../model';

export const usePluginSettingsForm = () => {
  const form = createForm({
    schema: PluginSettingsSchema,
    initialInput: {
      allowedHosts: [],
      allowedPaths: [],
    },
  });

  return form;
};

export const useResetPluginSettingsFormValues = (
  form: FormStore<typeof PluginSettingsSchema>,
  initialValues: Accessor<Partial<PluginSettingsSchemaInput> | undefined>,
) => {
  createEffect(
    on([() => form, initialValues], ([_formInstance, initial]) => {
      if (initial) {
        untrack(() => {
          setInput(form, { input: initial });
        });
      }
    }),
  );
};
