import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, setInput } from '@formisch/solid';
import { createEffect } from 'solid-js';

import { WindowSettingsSchema, type WindowSettingsSchemaInput } from '../model';

type WindowFormStore = FormStore<typeof WindowSettingsSchema>;

export const useWindowSettingsForm = (): WindowFormStore => {
  return createForm({ schema: WindowSettingsSchema });
};

export const useResetWindowSettingsFormValues = (
  form: WindowFormStore,
  initialValues: Accessor<Partial<WindowSettingsSchemaInput> | undefined>,
) => {
  createEffect(() => {
    const overrideWindowSettings = initialValues()?.overrideWindowSettings;

    if (overrideWindowSettings !== undefined) {
      setInput(form, {
        path: ['overrideWindowSettings'],
        input: overrideWindowSettings,
      });
    }
  });

  createEffect(() => {
    const resolution = initialValues()?.resolution;

    if (resolution?.width && resolution?.height) {
      setInput(form, {
        path: ['resolution', 'width'],
        input: resolution.width,
      });
      setInput(form, {
        path: ['resolution', 'height'],
        input: resolution.height,
      });
    }
  });
};
