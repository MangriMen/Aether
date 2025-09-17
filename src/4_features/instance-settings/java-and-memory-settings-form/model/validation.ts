import { z } from 'zod';

export const MemoryMaximumSchema = z.number().optional().nullable();

export const MemorySchema = z.object({
  maximum: MemoryMaximumSchema,
});

export const JavaAndMemorySettingsSchema = z.object({
  memory: MemorySchema,
  extraLaunchArgs: z.string().nullable(),
  customEnvVars: z.string().nullable(),
});

export type JavaAndMemorySettingsSchemaInput = z.input<
  typeof JavaAndMemorySettingsSchema
>;

export type JavaAndMemorySettingsSchemaOutput = z.output<
  typeof JavaAndMemorySettingsSchema
>;
