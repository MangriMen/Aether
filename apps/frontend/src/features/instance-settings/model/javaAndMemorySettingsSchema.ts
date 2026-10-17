import * as v from 'valibot';

import { createI18nError } from '@/shared/i18n';

import { stringToEnvVars } from './parseEnvVars';

const EnvVarsTransformation = v.pipe(
  v.string(),
  v.rawTransform(({ dataset, addIssue, NEVER }) => {
    const result = stringToEnvVars(dataset.value);

    if ('error' in result) {
      addIssue({
        message: createI18nError(result.error, { part: result.part }),
      });

      return NEVER;
    }

    return result.vars;
  }),
);

const MemoryMaximumSchema = v.number();

const MemorySchema = v.object({
  maximum: MemoryMaximumSchema,
});

export const JavaAndMemorySettingsSchema = v.object({
  memory: MemorySchema,
  launchArgs: v.string(),
  envVars: EnvVarsTransformation,
  overrideMemory: v.optional(v.boolean()),
  overrideLaunchArgs: v.optional(v.boolean()),
  overrideEnvVars: v.optional(v.boolean()),
});

export type JavaAndMemorySettingsSchemaInput = v.InferInput<
  typeof JavaAndMemorySettingsSchema
>;

export type JavaAndMemorySettingsSchemaOutput = v.InferOutput<
  typeof JavaAndMemorySettingsSchema
>;
