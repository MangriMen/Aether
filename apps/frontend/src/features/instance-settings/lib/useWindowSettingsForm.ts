import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, reset } from '@formisch/solid';

import {
  WindowSettingsSchema,
  type WindowSettingsSchemaInput,
  type WindowSettingsSchemaOutput,
} from '../model/windowSettingsSchema';
import { createFieldSync, reapplyFields } from './createFieldSync';

type WindowFormStore = FormStore<typeof WindowSettingsSchema>;

export const useWindowSettingsForm = (): WindowFormStore => {
  return createForm({ schema: WindowSettingsSchema });
};

export const useResetWindowSettingsFormValues = (
  form: WindowFormStore,
  initialValues: Accessor<Partial<WindowSettingsSchemaInput> | undefined>,
) => {
  const reapplyWidth = createFieldSync(
    () => initialValues()?.resolution?.width || undefined,
    (input) =>
      reset(form, { path: ['resolution', 'width'], initialInput: input }),
  );
  const reapplyHeight = createFieldSync(
    () => initialValues()?.resolution?.height || undefined,
    (input) =>
      reset(form, { path: ['resolution', 'height'], initialInput: input }),
  );

  const reappliers = {
    overrideWindowSettings: createFieldSync(
      () => initialValues()?.overrideWindowSettings,
      (input) =>
        reset(form, { path: ['overrideWindowSettings'], initialInput: input }),
    ),
    resolution: () => {
      reapplyWidth();
      reapplyHeight();
    },
  };

  return (values: Partial<WindowSettingsSchemaOutput>) =>
    reapplyFields(reappliers, values);
};
