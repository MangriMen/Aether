import * as v from 'valibot';

export const AllowedPathSchema = v.object({
  hostPath: v.string(),
  pluginPath: v.string(),
  readOnly: v.optional(v.boolean(), true),
});

export type AllowedPathValue = v.InferInput<typeof AllowedPathSchema>;
export type AllowedPathErrors = {
  hostPath?: string;
  pluginPath?: string;
};

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
    v.optional(v.array(AllowedPathSchema)),
    v.transform((value) => value ?? []),
  ),
});

export type PluginSettingsSchemaInput = v.InferInput<
  typeof PluginSettingsSchema
>;
export type PluginSettingsSchemaOutput = v.InferOutput<
  typeof PluginSettingsSchema
>;
