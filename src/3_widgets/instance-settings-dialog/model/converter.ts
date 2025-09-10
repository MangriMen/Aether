import type { Instance } from '@/entities/instances';
import type { PartialValues } from '@modular-forms/solid';
import type { WindowSchemaValuesInput } from './windowValidation';
import type { JavaAndMemorySettingsSchemaValuesInput } from './javaAndMemoryValidation';
import { envVarsToString, extraLaunchArgsToString } from '../lib';

export const instanceToWindowSettingsValues = (
  instance: Instance,
): PartialValues<WindowSchemaValuesInput> => {
  const resolution = instance.gameResolution;

  return {
    resolution: resolution
      ? {
          width: resolution[0].toString(),
          height: resolution[1].toString(),
        }
      : undefined,
  };
};

export const instanceToJavaAndMemorySettingsValues = (
  instance: Instance,
): PartialValues<JavaAndMemorySettingsSchemaValuesInput> => {
  const maximum = instance.memory?.maximum;
  const extraLaunchArgs = instance.extraLaunchArgs;
  const customEnvVars = instance.customEnvVars;

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
