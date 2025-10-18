import { z } from 'zod';

export const PluginSettingsSchema = z.object({
  allowedHosts: z
    .array(
      z
        .string()
        .trim()
        .refine((value) => value.length > 0, 'Not allowed empty domain name'),
    )
    .optional()
    .transform((value) => value ?? []),
  allowedPaths: z
    .array(
      z.tuple([
        z.string(), //.refine(() => false, 'HER'),
        z.string(), //.refine(() => false, 'PER'),
      ]),
    )
    .optional()
    .transform((value) => value ?? []),
});

export type PluginSettingsSchemaInput = z.input<typeof PluginSettingsSchema>;
export type PluginSettingsSchemaOutput = z.output<typeof PluginSettingsSchema>;
