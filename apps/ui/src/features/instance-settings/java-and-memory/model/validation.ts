import { z } from 'zod';

import { createI18nError } from '@/shared/lib';

export const MemoryMaximumSchema = z.number();

export const MemorySchema = z.object({
  maximum: MemoryMaximumSchema,
});

export const JavaAndMemorySettingsSchema = z.object({
  memory: MemorySchema,
  launchArgs: z.string(),
  envVars: z.string().transform((val, ctx) => {
    const parts = val
      .split(';')
      .map((p) => p.trim())
      .filter(Boolean);

    const result: [string, string][] = [];
    let hasError = false;

    for (const part of parts) {
      if (!part.includes('=')) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          message: createI18nError('invalidFormat', { part }),
        });
        hasError = true;
        continue;
      }

      const [key, ...valueParts] = part.split('=');
      const value = valueParts.join('=');

      if (!key?.trim()) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          message: createI18nError('emptyKey', { part }),
        });
        hasError = true;
        continue;
      }

      result.push([key.trim(), value.trim()]);
    }

    if (hasError) {
      return z.NEVER;
    }

    return result;
  }),

  overrideMemory: z.boolean().optional(),
  overrideLaunchArgs: z.boolean().optional(),
  overrideEnvVars: z.boolean().optional(),
});

export type JavaAndMemorySettingsSchemaInput = z.input<
  typeof JavaAndMemorySettingsSchema
>;

export type JavaAndMemorySettingsSchemaOutput = z.output<
  typeof JavaAndMemorySettingsSchema
>;
