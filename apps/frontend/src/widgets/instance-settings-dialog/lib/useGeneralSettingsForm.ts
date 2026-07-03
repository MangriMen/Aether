import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, setInput } from '@formisch/solid';
import { createEffect } from 'solid-js';

import {
  GeneralSettingsSchema,
  type GeneralSettingsSchemaInput,
} from '../model';

export const useGeneralSettingsForm = () => {
  const form = createForm({ schema: GeneralSettingsSchema });
  return form;
};

export const useResetGeneralSettingsFormValues = (
  form: FormStore<typeof GeneralSettingsSchema>,
  initialValues: Accessor<Partial<GeneralSettingsSchemaInput> | undefined>,
) => {
  createEffect(() => {
    const name = initialValues()?.name;

    if (name !== undefined) {
      setInput(form, { path: ['name'], input: name });
    }
  });
};
