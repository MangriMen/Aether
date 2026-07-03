import * as v from 'valibot';

export const NameSchema = v.pipe(
  v.string(),
  v.trim(),
  v.minLength(1, 'generalSettingsError.name.minLength'),
  v.maxLength(64, 'generalSettingsError.name.maxLength'),
);
// .regex(/^[a-zA-Z0-9]+$/, 'generalSettingsError.name.invalidSymbol')

export const GeneralSettingsSchema = v.object({
  name: NameSchema,
  icon: v.optional(v.nullable(v.string())),
});

export type GeneralSettingsSchemaInput = v.InferInput<
  typeof GeneralSettingsSchema
>;
export type GeneralSettingsSchemaOutput = v.InferOutput<
  typeof GeneralSettingsSchema
>;
