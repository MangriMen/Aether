import * as v from 'valibot';

import { createI18nError } from '@/shared/lib';

const EnvVarsTransformation = v.pipe(
  v.string(),
  v.transform((val) => {
    const parts = val
      .split(';')
      .map((p) => p.trim())
      .filter(Boolean);

    const result: [string, string][] = [];

    for (const part of parts) {
      if (!part.includes('=')) {
        throw new v.ValiError([
          {
            kind: 'validation',
            type: 'custom',
            input: val,
            expected: null,
            received: 'string',
            message: createI18nError('invalidFormat', { part }),
          },
        ]);
      }

      const [key, ...valueParts] = part.split('=');
      const value = valueParts.join('=');

      if (!key?.trim()) {
        throw new v.ValiError([
          {
            kind: 'validation',
            type: 'custom',
            input: val,
            expected: null,
            received: 'string',
            message: createI18nError('emptyKey', { part }),
          },
        ]);
      }

      result.push([key.trim(), value.trim()]);
    }

    return result;
  }),
);

export const MemoryMaximumSchema = v.number();

export const MemorySchema = v.object({
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
