import { z } from 'zod';

export const MemoryMaximumSchema = z.number().optional().nullable();

export const MemorySchema = z.object({
  maximum: MemoryMaximumSchema,
});

export type MemorySchemaValues = z.infer<typeof MemorySchema>;

export const JavaAndMemorySettingsSchemaRequired = z.object({
  memory: MemorySchema,
  extraLaunchArgs: z.string().nullable(),
  customEnvVars: z.string().nullable(),
});

export const JavaAndMemorySettingsSchema =
  JavaAndMemorySettingsSchemaRequired.extend({
    javaPath: z.string().nullable(),
  });

export type JavaAndMemorySettingsSchemaRequiredInput = z.infer<
  typeof JavaAndMemorySettingsSchemaRequired
>;

export type JavaAndMemorySettingsSchemaRequiredOutput = z.output<
  typeof JavaAndMemorySettingsSchemaRequired
>;

export type JavaAndMemorySettingsSchemaValuesInput = z.input<
  typeof JavaAndMemorySettingsSchema
>;

export type JavaAndMemorySettingsSchemaValuesOutput = z.output<
  typeof JavaAndMemorySettingsSchema
>;
