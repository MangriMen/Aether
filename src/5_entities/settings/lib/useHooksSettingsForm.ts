import type { FormStore, PartialValues } from '@modular-forms/solid';
import { createForm, setValues, zodForm } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';
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
    const pre_launch = initialValues()?.pre_launch;

    if (pre_launch) {
      setValues(form, {
        pre_launch,
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
    const post_exit = initialValues()?.post_exit;

    if (post_exit) {
      setValues(form, {
        post_exit,
      });
    }
  });
};
