import { z } from 'zod';

export const MemoryMaximumSchema = z.number().optional().nullable();

export const MemorySchema = z.object({
  maximum: MemoryMaximumSchema,
});

export const JavaAndMemorySettingsSchema = z.object({
  memory: MemorySchema,
  launchArgs: z.string().nullable(),
  envVars: z.string().nullable(),
});

export type JavaAndMemorySettingsSchemaInput = z.input<
  typeof JavaAndMemorySettingsSchema
>;

export type JavaAndMemorySettingsSchemaOutput = z.output<
  typeof JavaAndMemorySettingsSchema
>;
