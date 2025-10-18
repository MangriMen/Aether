import type { FormStore } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect } from 'solid-js';

import type { PluginSettingsSchemaValues } from '../model';

import { PluginSettingsSchema } from '../model';

export const usePluginSettingsForm = (): ReturnType<
  typeof createForm<PluginSettingsSchemaValues>
> => {
  const [form, components] = createForm<PluginSettingsSchemaValues>({
    validate: zodForm(PluginSettingsSchema),
    initialValues: {
      allowedHosts: [],
      allowedPaths: [],
    },
  });

  return [form, components];
};

export const useResetPluginSettingsFormValues = (
  form: FormStore<PluginSettingsSchemaValues>,
  initialValues: Accessor<PluginSettingsSchemaValues | undefined>,
) => {
  createEffect(() => {
    const initial = initialValues();

    if (initial) {
      setValues(form, initial);
    }
  });
};
