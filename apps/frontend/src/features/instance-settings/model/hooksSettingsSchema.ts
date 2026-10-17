import * as v from 'valibot';

export const HooksSettingsSchema = v.object({
  preLaunch: v.string(),
  wrapper: v.string(),
  postExit: v.string(),
  overrideHooks: v.optional(v.boolean()),
});

export type HooksSettingsSchemaInput = v.InferInput<typeof HooksSettingsSchema>;
export type HooksSettingsSchemaOutput = v.InferOutput<
  typeof HooksSettingsSchema
>;
