import z from 'zod';

export const NameSchema = z.string();
// .trim()
// .min(1, 'generalSettingsError.name.minLength')
// .max(90, 'generalSettingsError.name.maxLength')
// .regex(/^[a-zA-Z0-9]+$/, 'generalSettingsError.name.invalidSymbol')

export const GeneralSettingsSchema = z.object({
  name: NameSchema,
});

export type GeneralSettingsSchemaInput = z.input<typeof GeneralSettingsSchema>;
export type GeneralSettingsSchemaOutput = z.output<
  typeof GeneralSettingsSchema
>;
