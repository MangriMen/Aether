import type { FormStore, PartialValues } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect } from 'solid-js';

import {
  HooksSettingsSchema,
  type HooksSettingsSchemaValuesInput,
} from '../model';

export const useHooksSettingsForm = (): ReturnType<
  typeof createForm<HooksSettingsSchemaValuesInput>
> => {
  const [form, components] = createForm({
    validate: zodForm(HooksSettingsSchema),
  });

  return [form, components];
};

export const useResetHooksSettingsFormValues = (
  form: FormStore<HooksSettingsSchemaValuesInput>,
  initialValues: Accessor<
    PartialValues<HooksSettingsSchemaValuesInput> | undefined
  >,
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
