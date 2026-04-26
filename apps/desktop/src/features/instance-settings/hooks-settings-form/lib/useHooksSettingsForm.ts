import type { FormStore, PartialValues } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect } from 'solid-js';

import { HooksSettingsSchema, type HooksSettingsSchemaInput } from '../model';

export const useHooksSettingsForm = (): ReturnType<
  typeof createForm<HooksSettingsSchemaInput>
> => {
  const [form, components] = createForm({
    validate: zodForm(HooksSettingsSchema),
  });

  return [form, components];
};

export const useResetHooksSettingsFormValues = (
  form: FormStore<HooksSettingsSchemaInput>,
  initialValues: Accessor<PartialValues<HooksSettingsSchemaInput> | undefined>,
) => {
  createEffect(() => {
    const preLaunch = initialValues()?.preLaunch;

    if (preLaunch) {
      setValues(form, {
        preLaunch: preLaunch,
      });
    }
  });

  createEffect(() => {
    const wrapper = initialValues()?.wrapper;

    if (wrapper) {
      setValues(form, {
        wrapper,
      });
    }
  });

  createEffect(() => {
    const post_exit = initialValues()?.postExit;

    if (post_exit) {
      setValues(form, {
        postExit: post_exit,
      });
    }
  });
};
