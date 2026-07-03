import * as v from 'valibot';

export const PluginSettingsSchema = v.object({
  allowedHosts: v.pipe(
    v.optional(
      v.array(
        v.pipe(
          v.string(),
          v.trim(),
          v.minLength(1, 'Not allowed empty domain name'),
        ),
      ),
    ),
    v.transform((value) => value ?? []),
  ),
  allowedPaths: v.pipe(
    v.optional(v.array(v.tuple([v.string(), v.string()]))),
    v.transform((value) => value ?? []),
  ),
});

export type PluginSettingsSchemaInput = v.InferInput<
  typeof PluginSettingsSchema
>;
export type PluginSettingsSchemaOutput = v.InferOutput<
  typeof PluginSettingsSchema
>;
