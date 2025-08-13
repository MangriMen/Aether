import z from 'zod';

export const GeneralSettingsSchema = z.object({
  name: z.string(),
  // .trim()
  // .min(1, 'generalSettingsError.name.minLength')
  // .max(90, 'generalSettingsError.name.maxLength')
  // .regex(/^[a-zA-Z0-9]+$/, 'generalSettingsError.name.invalidSymbol'),
});

export type GeneralSettingsSchemaValues = z.infer<typeof GeneralSettingsSchema>;
