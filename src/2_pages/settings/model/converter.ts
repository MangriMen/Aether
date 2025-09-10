import type { HooksSettingsSchemaValuesInput } from '@/entities/settings';
import type { GlobalInstanceSettings } from '@/entities/settings/model/globalInstanceSettings';
import type {
  JavaAndMemorySettingsSchemaValuesInput,
  WindowSchemaValuesInput,
} from '@/widgets/instance-settings-dialog';
import {
  envVarsToString,
  extraLaunchArgsToString,
} from '@/widgets/instance-settings-dialog/lib';
import type { PartialValues } from '@modular-forms/solid';

export const instanceSettingsToWindowSettingsValues = (
  settings: GlobalInstanceSettings,
): PartialValues<WindowSchemaValuesInput> => {
  const resolution = settings.gameResolution;

  return {
    resolution: resolution
      ? {
          width: resolution[0].toString(),
          height: resolution[1].toString(),
        }
      : undefined,
  };
};

export const instanceSettingsToJavaAndMemorySettingsValues = (
  settings: GlobalInstanceSettings,
): PartialValues<JavaAndMemorySettingsSchemaValuesInput> => {
  const maximum = settings.memory.maximum;
  const extraLaunchArgs = settings.extraLaunchArgs;
  const customEnvVars = settings.customEnvVars;

  return {
    memory: {
      maximum,
    },
    extraLaunchArgs: extraLaunchArgs
      ? extraLaunchArgsToString(extraLaunchArgs)
      : null,
    customEnvVars: customEnvVars ? envVarsToString(customEnvVars) : null,
  };
};

export const instanceSettingsToHooksSettingsValues = (
  settings: GlobalInstanceSettings,
): PartialValues<HooksSettingsSchemaValuesInput> => {
  const hooks = settings.hooks;

  return { ...hooks };
};
