import type { FormStore } from '@formisch/solid';
import type { Accessor } from 'solid-js';

import { createForm, setInput } from '@formisch/solid';
import { createEffect } from 'solid-js';

import {
  JavaAndMemorySettingsSchema,
  type JavaAndMemorySettingsSchemaInput,
} from '../model';

export const useJavaAndMemorySettingsForm = () => {
  const form = createForm({ schema: JavaAndMemorySettingsSchema });
  return form;
};

export const useResetJavaAndMemorySettingsForm = (
  form: FormStore<typeof JavaAndMemorySettingsSchema>,
  settings: Accessor<Partial<JavaAndMemorySettingsSchemaInput> | undefined>,
) => {
  createEffect(() => {
    const overrideMemory = settings()?.overrideMemory;

    if (overrideMemory !== undefined) {
      setInput(form, { path: ['overrideMemory'], input: overrideMemory });
    }
  });

  createEffect(() => {
    const overrideLaunchArgs = settings()?.overrideLaunchArgs;

    if (overrideLaunchArgs !== undefined) {
      setInput(form, {
        path: ['overrideLaunchArgs'],
        input: overrideLaunchArgs,
      });
    }
  });

  createEffect(() => {
    const overrideEnvVars = settings()?.overrideEnvVars;

    if (overrideEnvVars !== undefined) {
      setInput(form, { path: ['overrideEnvVars'], input: overrideEnvVars });
    }
  });

  createEffect(() => {
    const maximum = settings()?.memory?.maximum;

    if (maximum !== undefined) {
      setInput(form, { path: ['memory', 'maximum'], input: maximum });
    }
  });

  createEffect(() => {
    const launchArgs = settings()?.launchArgs;

    if (launchArgs !== undefined) {
      setInput(form, { path: ['launchArgs'], input: launchArgs });
    }
  });

  createEffect(() => {
    const envVars = settings()?.envVars;

    if (envVars !== undefined) {
      setInput(form, { path: ['envVars'], input: envVars });
    }
  });
};
