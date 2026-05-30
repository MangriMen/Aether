import type { FormStore, PartialValues } from '@modular-forms/solid';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect, type Accessor } from 'solid-js';

import type { GeneralSettingsSchemaInput } from '../model';

import { GeneralSettingsSchema } from '../model';

export const useGeneralSettingsForm = (): ReturnType<
  typeof createForm<GeneralSettingsSchemaInput>
> => {
  const [form, components] = createForm({
    validate: zodForm(GeneralSettingsSchema),
  });

  return [form, components];
};

export const useResetGeneralSettingsFormValues = (
  form: FormStore<GeneralSettingsSchemaInput>,
  initialValues: Accessor<
    PartialValues<GeneralSettingsSchemaInput> | undefined
  >,
) => {
  createEffect(() => {
    const name = initialValues()?.name;

    if (name) {
      setValues(form, { name });
    }
  });
};
