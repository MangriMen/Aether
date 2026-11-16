import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, setInput } from '@formisch/solid';
import { createEffect, createMemo } from 'solid-js';

import { FIELDS_CONFIG } from '../model/hooksFields';
import {
  HooksSettingsSchema,
  type HooksSettingsSchemaInput,
} from '../model/hooksSettingsSchema';

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

  // Saving one hook refreshes the values of all of them; only a changed stored command may reset
  // its field, or the refresh would wipe a command the user is still fixing (an invalid one is
  // not saved).
  for (const { key } of FIELDS_CONFIG) {
    const command = createMemo(() => initialValues()?.[key]);

    createEffect(() => {
      const value = command();

      if (value !== undefined) {
        setInput(form, { path: [key], input: value });
      }
    });
  }
};
