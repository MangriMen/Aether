import type { PartialValues } from '@modular-forms/solid';

import type {
  EditInstance,
  EditInstanceSettings,
  Instance,
  InstanceSettings,
} from '@/entities/instances';
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
  launchArgsToString,
  stringToEnvVars,
  stringToLaunchArgs,
} from '@/widgets/instance-settings-dialog/lib';

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

  return { name: instance.name };
};

export const instanceSettingsToWindowSettingsValues = (
  settings: InstanceSettings | undefined,
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

export const instanceSettingsToJavaAndMemorySettingsValues = (
  settings: InstanceSettings | undefined,
): PartialValues<JavaAndMemorySettingsSchemaInput> | undefined => {
  if (!settings) {
    return;
  }

  const { memory } = settings;

  const maximum = memory === null ? null : memory?.maximum;
  const launchArgs = settings.launchArgs;
  const envVars = settings.envVars;

  return {
    memory: {
      maximum,
    },
    launchArgs: launchArgs ? launchArgsToString(launchArgs) : null,
    envVars: envVars ? envVarsToString(envVars) : null,
  };
};

export const instanceSettingsToHooksSettingsValues = (
  settings: InstanceSettings | undefined,
): PartialValues<HooksSettingsSchemaInput> | undefined => {
  if (!settings) {
    return;
  }

  return {
    preLaunch: settings.hooks?.preLaunch,
    wrapper: settings.hooks?.wrapper,
    postExit: settings.hooks?.postExit,
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
  values: Partial<WindowSettingsSchemaOutput>,
): EditInstanceSettings => {
  const dto: EditInstanceSettings = {};

  if (values.resolution !== undefined) {
    dto.gameResolution =
      values.resolution === null
        ? null
        : [values.resolution.width, values.resolution.height];
  }

  return dto;
};

export const javaAndMemorySettingsValuesToEditInstanceSettings = (
  values: Partial<JavaAndMemorySettingsSchemaOutput>,
): EditInstanceSettings => {
  const dto: EditInstanceSettings = {};

  const { memory } = values;

  if (memory !== undefined && memory.maximum !== undefined) {
    dto.memory = memory.maximum !== null ? { maximum: memory.maximum } : null;
  }

  if (values.launchArgs !== undefined) {
    dto.launchArgs =
      values.launchArgs === null ? null : stringToLaunchArgs(values.launchArgs);
  }

  if (values.envVars !== undefined) {
    dto.envVars =
      values.envVars === null ? null : stringToEnvVars(values.envVars);
  }

  return dto;
};

export const hooksSettingsValuesToEditInstanceSettings = (
  values: Partial<HooksSettingsSchemaOutput>,
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
