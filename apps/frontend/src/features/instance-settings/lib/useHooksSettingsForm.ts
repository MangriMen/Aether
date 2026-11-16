import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, reset } from '@formisch/solid';

import { FIELDS_CONFIG } from '../model/hooksFields';
import {
  HooksSettingsSchema,
  type HooksSettingsSchemaInput,
  type HooksSettingsSchemaOutput,
} from '../model/hooksSettingsSchema';
import { createFieldSync, reapplyFields } from './createFieldSync';

export const useHooksSettingsForm = () => {
  const form = createForm({ schema: HooksSettingsSchema });
  return form;
};

export const useResetHooksSettingsFormValues = (
  form: FormStore<typeof HooksSettingsSchema>,
  initialValues: Accessor<Partial<HooksSettingsSchemaInput> | undefined>,
) => {
  const reappliers: Partial<
    Record<keyof HooksSettingsSchemaOutput, () => void>
  > = {
    overrideHooks: createFieldSync(
      () => initialValues()?.overrideHooks,
      (input) => reset(form, { path: ['overrideHooks'], initialInput: input }),
    ),
  };

  for (const { key } of FIELDS_CONFIG) {
    reappliers[key] = createFieldSync(
      () => initialValues()?.[key],
      (input) => reset(form, { path: [key], initialInput: input }),
    );
  }

  return (values: Partial<HooksSettingsSchemaOutput>) =>
    reapplyFields(reappliers, values);
};
