import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, setInput } from '@formisch/solid';
import { createEffect, createMemo } from 'solid-js';

import {
  GeneralSettingsSchema,
  type GeneralSettingsSchemaInput,
} from '../model/generalValidation';

export const useGeneralSettingsForm = () => {
  const form = createForm({ schema: GeneralSettingsSchema });
  return form;
};

export const useResetGeneralSettingsFormValues = (
  form: FormStore<typeof GeneralSettingsSchema>,
  initialValues: Accessor<Partial<GeneralSettingsSchemaInput> | undefined>,
) => {
  // The values object is rebuilt on every instance refresh; only a changed name may reset the
  // field, or a refresh would overwrite what the user is typing.
  const name = createMemo(() => initialValues()?.name);

  createEffect(() => {
    const value = name();

    if (value !== undefined) {
      setInput(form, { path: ['name'], input: value });
    }
  });
};
