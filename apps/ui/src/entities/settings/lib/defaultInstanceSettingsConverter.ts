import type { PartialValues } from '@modular-forms/solid';

import type {
  HooksSettingsSchemaInput,
  HooksSettingsSchemaOutput,
} from '@/features/instance-settings/hooks';
import type {
  JavaAndMemorySettingsSchemaInput,
  JavaAndMemorySettingsSchemaOutput,
} from '@/features/instance-settings/java-and-memory';
import type {
  WindowSettingsSchemaInput,
  WindowSettingsSchemaOutput,
} from '@/features/instance-settings/window';

import {
  envVarsToString,
  launchArgsToString,
  stringToLaunchArgs,
  type DefaultInstanceSettings,
  type EditDefaultInstanceSettings,
} from '@/entities/settings';

export const defaultInstanceSettingsToWindowSettingsValues = (
  settings: DefaultInstanceSettings | undefined,
): PartialValues<WindowSettingsSchemaInput> | undefined => {
  if (!settings) {
    return;
  }

  const resolution = settings.window.gameResolution;

  return {
    resolution: resolution
      ? {
          width: resolution[0].toString(),
          height: resolution[1].toString(),
        }
      : undefined,
    forceFullscreen: settings.window.forceFullscreen,
  };
};

export const defaultInstanceSettingsToJavaAndMemorySettingsValues = (
  settings: DefaultInstanceSettings | undefined,
): PartialValues<JavaAndMemorySettingsSchemaInput> | undefined => {
  if (!settings) {
    return;
  }

  const maximum = settings.memory.maximum;
  const launchArgs = settings.launchArgs;
  const envVars = settings.envVars;

  return {
    memory: {
      maximum,
    },
    launchArgs: launchArgsToString(launchArgs),
    envVars: envVarsToString(envVars),
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
    dto.window = {
      forceFullscreen: false,
      gameResolution: [values.resolution.width, values.resolution.height],
    };
  }

  return dto;
};

export const javaAndMemorySettingsValuesToEditDefaultInstanceSettings = (
  values: Partial<JavaAndMemorySettingsSchemaOutput>,
): EditDefaultInstanceSettings => {
  const dto: EditDefaultInstanceSettings = {};

  if (values.memory?.maximum !== undefined) {
    dto.memory = { maximum: values.memory.maximum };
  }

  if (values.launchArgs !== undefined) {
    dto.launchArgs = stringToLaunchArgs(values.launchArgs);
  }

  if (values.envVars !== undefined) {
    dto.envVars = values.envVars;
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
