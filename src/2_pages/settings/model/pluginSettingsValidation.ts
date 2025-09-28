import { z } from 'zod';

export const PluginSettingsSchema = z.object({
  allowedHosts: z
    .string()
    .array()
    .optional()
    .transform((value) => value ?? []),
  allowedPaths: z
    .tuple([z.string(), z.string()])
    .array()
    .optional()
    .transform((value) => value ?? []),
});

export type PluginSettingsSchemaValues = z.infer<typeof PluginSettingsSchema>;
