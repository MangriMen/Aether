import { type PartialValues } from '@modular-forms/solid';

import type {
  EditInstance,
  EditInstanceSettings,
  Instance,
  InstanceSettings,
} from '@/entities/instances';
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
} from '@/entities/settings';

import type {
  GeneralSettingsSchemaInput,
  GeneralSettingsSchemaOutput,
} from './generalValidation';

export const instanceToGeneralSettingsValues = (
  instance: Instance | undefined,
): PartialValues<GeneralSettingsSchemaInput> | undefined => {
  if (!instance) {
    return;
  }

  return { name: instance.name, icon: instance.iconPath ?? undefined };
};

export const instanceSettingsToWindowSettingsValues = (
  settings: InstanceSettings | undefined,
): PartialValues<WindowSettingsSchemaInput> | undefined => {
  if (!settings) {
    return;
  }

  const resolution = settings.window.data.gameResolution;

  return {
    resolution: resolution
      ? {
          width: resolution[0].toString(),
          height: resolution[1].toString(),
        }
      : undefined,
    forceFullscreen: settings.window.data.forceFullscreen,
    overrideWindowSettings: settings.window.isActive,
  };
};

export const instanceSettingsToJavaAndMemorySettingsValues = (
  settings: InstanceSettings | undefined,
): PartialValues<JavaAndMemorySettingsSchemaInput> | undefined => {
  if (!settings) {
    return;
  }

  const { memory } = settings;

  const maximum = memory.data.maximum;
  const launchArgs = settings.launchArgs.data;
  const envVars = settings.envVars.data;

  return {
    memory: {
      maximum,
    },
    launchArgs: launchArgsToString(launchArgs),
    envVars: envVarsToString(envVars),
    overrideMemory: settings.memory.isActive,
    overrideEnvVars: settings.envVars.isActive,
    overrideLaunchArgs: settings.launchArgs.isActive,
  };
};

export const instanceSettingsToHooksSettingsValues = (
  settings: InstanceSettings | undefined,
): PartialValues<HooksSettingsSchemaInput> | undefined => {
  if (!settings) {
    return;
  }

  return {
    preLaunch: settings.hooks?.data.preLaunch,
    wrapper: settings.hooks?.data.wrapper,
    postExit: settings.hooks?.data.postExit,
    overrideHooks: settings.hooks?.isActive,
  };
};

export const generalSettingsValuesToEditInstance = (
  values: Partial<GeneralSettingsSchemaOutput>,
) => {
  const dto: EditInstance = {};

  if (values.name !== undefined) {
    dto.name = values.name;
  }

  return dto;
};

export const windowSettingsValuesToEditInstanceSettings = (
  instance: Instance,
  values: Partial<WindowSettingsSchemaOutput>,
): EditInstanceSettings => {
  const dto: EditInstanceSettings = {};

  if (values.forceFullscreen !== undefined || values.resolution !== undefined) {
    const forceFullscreen =
      values.forceFullscreen ?? instance.window.data.forceFullscreen;

    const gameResolution: [number, number] =
      values.resolution !== undefined
        ? [values.resolution.width, values.resolution.height]
        : instance.window.data.gameResolution;

    dto.window = {
      forceFullscreen,
      gameResolution,
    };
  }

  if (values.overrideWindowSettings !== undefined) {
    dto.overrideWindowSettings = values.overrideWindowSettings;
  }

  return dto;
};

export const javaAndMemorySettingsValuesToEditInstanceSettings = (
  values: Partial<JavaAndMemorySettingsSchemaOutput>,
): EditInstanceSettings => {
  const dto: EditInstanceSettings = {};

  if (values.memory !== undefined) {
    dto.memory = { maximum: values.memory.maximum };
  }

  if (values.launchArgs !== undefined) {
    dto.launchArgs = stringToLaunchArgs(values.launchArgs);
  }

  if (values.envVars !== undefined) {
    dto.envVars = values.envVars;
  }

  if (values.overrideMemory !== undefined) {
    dto.overrideMemory = values.overrideMemory;
  }

  if (values.overrideLaunchArgs !== undefined) {
    dto.overrideLaunchArgs = values.overrideLaunchArgs;
  }

  if (values.overrideEnvVars !== undefined) {
    dto.overrideEnvVars = values.overrideEnvVars;
  }

  return dto;
};

export const hooksSettingsValuesToEditInstanceSettings = (
  values: Partial<HooksSettingsSchemaOutput>,
): EditInstanceSettings => {
  const dto: EditInstanceSettings = {};

  if (values.preLaunch !== undefined) {
    dto.hooks = {
      ...dto.hooks,
      preLaunch: values.preLaunch,
    };
  }

  if (values.wrapper !== undefined) {
    dto.hooks = { ...dto.hooks, wrapper: values.wrapper };
  }

  if (values.postExit !== undefined) {
    dto.hooks = { ...dto.hooks, postExit: values.postExit };
  }

  if (values.overrideHooks !== undefined) {
    dto.overrideHooks = values.overrideHooks;
  }

  return dto;
};
