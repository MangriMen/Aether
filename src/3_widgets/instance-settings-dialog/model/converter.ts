import type {
  EditInstanceSettings,
  InstanceSettings,
} from '@/entities/instances';
import type {
  HooksSettingsSchemaValuesInput,
  HooksSettingsSchemaValuesOutput,
} from '@/features/instance-settings/hooks-settings-form';
import type {
  WindowSchemaValuesInput,
  WindowSchemaValuesOutput,
} from '@/features/instance-settings/window-settings-form';
import type {
  JavaAndMemorySettingsSchemaValuesInput,
  JavaAndMemorySettingsSchemaValuesOutput,
} from '@/widgets/instance-settings-dialog';
import {
  envVarsToString,
  extraLaunchArgsToString,
  stringToEnvVars,
  stringToExtraLaunchArgs,
} from '@/widgets/instance-settings-dialog/lib';
import type { PartialValues } from '@modular-forms/solid';

export const instanceSettingsToWindowSettingsValues = (
  settings: InstanceSettings | undefined,
): PartialValues<WindowSchemaValuesInput> | undefined => {
  if (!settings) {
    return;
  }

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
  settings: InstanceSettings | undefined,
): PartialValues<JavaAndMemorySettingsSchemaValuesInput> | undefined => {
  if (!settings) {
    return;
  }

  const maximum = settings.memory?.maximum;
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
  settings: InstanceSettings | undefined,
): PartialValues<HooksSettingsSchemaValuesInput> | undefined => {
  if (!settings) {
    return;
  }

  return {
    preLaunch: settings.hooks?.preLaunch,
    wrapper: settings.hooks?.wrapper,
    postExit: settings.hooks?.postExit,
  };
};

export const windowSettingsValuesToEditInstanceSettings = (
  values: Partial<WindowSchemaValuesOutput>,
): EditInstanceSettings => {
  const dto: EditInstanceSettings = {};

  if (values.resolution) {
    dto.gameResolution = [values.resolution.width, values.resolution.height];
  }

  return dto;
};

export const javaAndMemorySettingsValuesToEditInstanceSettings = (
  values: Partial<JavaAndMemorySettingsSchemaValuesOutput>,
): EditInstanceSettings => {
  const dto: EditInstanceSettings = {};

  if (values.memory?.maximum) {
    dto.memory = { maximum: values.memory.maximum };
  }

  if (values.extraLaunchArgs) {
    dto.extraLaunchArgs = stringToExtraLaunchArgs(values.extraLaunchArgs);
  }

  if (values.customEnvVars) {
    dto.customEnvVars = stringToEnvVars(values.customEnvVars);
  }

  return dto;
};

export const hooksSettingsValuesToEditInstanceSettings = (
  values: Partial<HooksSettingsSchemaValuesOutput>,
): EditInstanceSettings => {
  const dto: EditInstanceSettings = {};

  if (
    values.preLaunch !== undefined ||
    values.wrapper !== undefined ||
    values.postExit !== undefined
  ) {
    dto.hooks = {
      preLaunch: values.preLaunch,
      wrapper: values.wrapper,
      postExit: values.postExit,
    };
  }

  return dto;
};
