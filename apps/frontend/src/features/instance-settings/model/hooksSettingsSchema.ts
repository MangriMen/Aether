import * as v from 'valibot';

import { hasUnclosedQuote, normalizeHookCommand } from './parseCommandLine';

export const UNCLOSED_QUOTE_ERROR =
  'instanceSettings.hookSettings.unclosedQuote';

/** An empty command stays valid: the hook is not set. */
const HookCommandSchema = v.pipe(
  v.string(),
  v.transform(normalizeHookCommand),
  v.check((value) => !hasUnclosedQuote(value), UNCLOSED_QUOTE_ERROR),
);

export const HooksSettingsSchema = v.object({
  preLaunch: HookCommandSchema,
  wrapper: HookCommandSchema,
  postExit: HookCommandSchema,
  overrideHooks: v.optional(v.boolean()),
});

export type HooksSettingsSchemaInput = v.InferInput<typeof HooksSettingsSchema>;
export type HooksSettingsSchemaOutput = v.InferOutput<
  typeof HooksSettingsSchema
>;
