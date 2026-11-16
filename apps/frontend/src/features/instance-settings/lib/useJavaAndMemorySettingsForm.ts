import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, reset } from '@formisch/solid';

import {
  JavaAndMemorySettingsSchema,
  type JavaAndMemorySettingsSchemaInput,
  type JavaAndMemorySettingsSchemaOutput,
} from '../model/javaAndMemorySettingsSchema';
import { createFieldSync, reapplyFields } from './createFieldSync';

export const useJavaAndMemorySettingsForm = () => {
  const form = createForm({ schema: JavaAndMemorySettingsSchema });
  return form;
};

export const useResetJavaAndMemorySettingsForm = (
  form: FormStore<typeof JavaAndMemorySettingsSchema>,
  settings: Accessor<Partial<JavaAndMemorySettingsSchemaInput> | undefined>,
) => {
  const reappliers = {
    overrideMemory: createFieldSync(
      () => settings()?.overrideMemory,
      (input) => reset(form, { path: ['overrideMemory'], initialInput: input }),
    ),
    overrideLaunchArgs: createFieldSync(
      () => settings()?.overrideLaunchArgs,
      (input) =>
        reset(form, { path: ['overrideLaunchArgs'], initialInput: input }),
    ),
    overrideEnvVars: createFieldSync(
      () => settings()?.overrideEnvVars,
      (input) =>
        reset(form, { path: ['overrideEnvVars'], initialInput: input }),
    ),
    memory: createFieldSync(
      () => settings()?.memory?.maximum,
      (input) =>
        reset(form, { path: ['memory', 'maximum'], initialInput: input }),
    ),
    launchArgs: createFieldSync(
      () => settings()?.launchArgs,
      (input) => reset(form, { path: ['launchArgs'], initialInput: input }),
    ),
    envVars: createFieldSync(
      () => settings()?.envVars,
      (input) => reset(form, { path: ['envVars'], initialInput: input }),
    ),
  };

  return (values: Partial<JavaAndMemorySettingsSchemaOutput>) =>
    reapplyFields(reappliers, values);
};
