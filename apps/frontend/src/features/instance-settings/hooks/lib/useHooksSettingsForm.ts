import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, setInput } from '@formisch/solid';
import { createEffect } from 'solid-js';

import { HooksSettingsSchema, type HooksSettingsSchemaInput } from '../model';

export const useHooksSettingsForm = () => {
  const form = createForm({ schema: HooksSettingsSchema });
  return form;
};

export const useResetHooksSettingsFormValues = (
  form: FormStore<typeof HooksSettingsSchema>,
  initialValues: Accessor<Partial<HooksSettingsSchemaInput> | undefined>,
) => {
  createEffect(() => {
    const overrideHooks = initialValues()?.overrideHooks;

    if (overrideHooks !== undefined) {
      setInput(form, { path: ['overrideHooks'], input: overrideHooks });
    }
  });

  createEffect(() => {
    const preLaunch = initialValues()?.preLaunch;

    if (preLaunch !== undefined) {
      setInput(form, { path: ['preLaunch'], input: preLaunch });
    }
  });

  createEffect(() => {
    const wrapper = initialValues()?.wrapper;

    if (wrapper !== undefined) {
      setInput(form, { path: ['wrapper'], input: wrapper });
    }
  });

  createEffect(() => {
    const postExit = initialValues()?.postExit;

    if (postExit !== undefined) {
      setInput(form, { path: ['postExit'], input: postExit });
    }
  });
};
