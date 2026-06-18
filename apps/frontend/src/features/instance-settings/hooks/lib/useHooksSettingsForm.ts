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
    const overrideHooks = initialValues()?.overrideHooks;

    if (overrideHooks !== undefined) {
      setValues(form, {
        overrideHooks,
      });
    }
  });

  createEffect(() => {
    const preLaunch = initialValues()?.preLaunch;

    if (preLaunch !== undefined) {
      setValues(form, {
        preLaunch,
      });
    }
  });

  createEffect(() => {
    const wrapper = initialValues()?.wrapper;

    if (wrapper !== undefined) {
      setValues(form, {
        wrapper,
      });
    }
  });

  createEffect(() => {
    const postExit = initialValues()?.postExit;

    if (postExit !== undefined) {
      setValues(form, {
        postExit,
      });
    }
  });
};
