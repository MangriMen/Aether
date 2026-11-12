import * as v from 'valibot';

import { ALLOWED_PATH_REJECTIONS } from '@/features/plugins';

const AllowedPathSchema = v.object({
  hostPath: v.string(),
  pluginPath: v.string(),
  readOnly: v.optional(v.boolean(), true),
  // Set on a saved folder the launcher did not read, and why. Such a folder is shown but never
  // given to the plugin; editing it makes it an ordinary one.
  ignoredReason: v.optional(v.picklist(ALLOWED_PATH_REJECTIONS)),
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
        v.pipe(v.string(), v.trim(), v.minLength(1, 'validation.hostRequired')),
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
