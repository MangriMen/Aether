import { z } from 'zod';

export const MemoryMaximumSchema = z.number().optional().nullable();

export const MemorySchema = z.object({
  maximum: MemoryMaximumSchema,
});

export type MemorySchemaValues = z.infer<typeof MemorySchema>;

export const JavaAndMemorySettingsSchema = z.object({
  customEnvVars: z.string().nullable(),
  extraLaunchArgs: z.string().nullable(),
  javaPath: z.string().nullable(),
  memory: MemorySchema,
});

export type JavaAndMemorySettingsSchemaValues = z.infer<
  typeof JavaAndMemorySettingsSchema
>;
