import * as v from 'valibot';

import {
  type CommandLineError,
  getHookCommandError,
  normalizeHookCommand,
} from './parseCommandLine';

/** The translation key of each reason the core refuses a hook command for. */
export const HOOK_COMMAND_ERRORS = {
  EMPTY: 'instanceSettings.hookSettings.emptyProgram',
  UNCLOSED_QUOTE: 'instanceSettings.hookSettings.unclosedQuote',
} as const satisfies Record<CommandLineError, string>;

/** An empty command stays valid: the hook is not set. */
const HookCommandSchema = v.pipe(
  v.string(),
  v.transform(normalizeHookCommand),
  v.rawCheck(({ dataset, addIssue }) => {
    const error = dataset.typed
      ? getHookCommandError(dataset.value)
      : undefined;

    if (error) {
      addIssue({ message: HOOK_COMMAND_ERRORS[error] });
    }
  }),
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
