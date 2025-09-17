import type { PartialValues } from '@modular-forms/solid';

import type {
  DefaultInstanceSettings,
  EditDefaultInstanceSettings,
} from '@/entities/settings/model/defaultInstanceSettings';
import type {
  HooksSettingsSchemaInput,
  HooksSettingsSchemaOutput,
} from '@/features/instance-settings/hooks-settings-form';
import type {
  JavaAndMemorySettingsSchemaInput,
  JavaAndMemorySettingsSchemaOutput,
} from '@/features/instance-settings/java-and-memory-settings-form';
import type {
  WindowSettingsSchemaInput,
  WindowSettingsSchemaOutput,
} from '@/features/instance-settings/window-settings-form';

import {
  envVarsToString,
  extraLaunchArgsToString,
  stringToEnvVars,
  stringToExtraLaunchArgs,
} from '@/widgets/instance-settings-dialog/lib';

export const defaultInstanceSettingsToWindowSettingsValues = (
  settings: DefaultInstanceSettings | undefined,
): PartialValues<WindowSettingsSchemaInput> | undefined => {
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

export const defaultInstanceSettingsToJavaAndMemorySettingsValues = (
  settings: DefaultInstanceSettings | undefined,
): PartialValues<JavaAndMemorySettingsSchemaInput> | undefined => {
  if (!settings) {
    return;
  }

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

export const defaultInstanceSettingsToHooksSettingsValues = (
  settings: DefaultInstanceSettings | undefined,
): PartialValues<HooksSettingsSchemaInput> | undefined => {
  if (!settings) {
    return;
  }

  const { preLaunch, wrapper, postExit } = settings.hooks;

  return {
    preLaunch,
    wrapper,
    postExit,
  };
};

export const windowSettingsValuesToEditDefaultInstanceSettings = (
  values: Partial<WindowSettingsSchemaOutput>,
): EditDefaultInstanceSettings => {
  const dto: EditDefaultInstanceSettings = {};

  if (values.resolution) {
    dto.gameResolution = [values.resolution.width, values.resolution.height];
  }

  return dto;
};

export const javaAndMemorySettingsValuesToEditDefaultInstanceSettings = (
  values: Partial<JavaAndMemorySettingsSchemaOutput>,
): EditDefaultInstanceSettings => {
  const dto: EditDefaultInstanceSettings = {};

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

export const hooksSettingsValuesToEditDefaultInstanceSettings = (
  values: Partial<HooksSettingsSchemaOutput>,
): EditDefaultInstanceSettings => {
  const dto: EditDefaultInstanceSettings = {};

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
