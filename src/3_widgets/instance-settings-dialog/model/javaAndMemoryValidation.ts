import { z } from 'zod';

export const MemoryMaximumSchema = z.number().optional().nullable();

export const MemorySchema = z.object({
  maximum: MemoryMaximumSchema,
});

export type MemorySchemaValues = z.infer<typeof MemorySchema>;

export const JavaAndMemorySettingsSchema = z.object({
  javaPath: z.string().nullable(),
  memory: MemorySchema,
  extraLaunchArgs: z.string().nullable(),
  customEnvVars: z.string().nullable(),
});

export type JavaAndMemorySettingsSchemaValues = z.infer<
  typeof JavaAndMemorySettingsSchema
>;
