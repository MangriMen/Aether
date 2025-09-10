import { z } from 'zod';

export const HooksSettingsSchema = z.object({
  pre_launch: z.string(),
  wrapper: z.string(),
  post_exit: z.string(),
});

export type HooksSettingsSchemaValuesInput = z.input<
  typeof HooksSettingsSchema
>;
export type HooksSettingsSchemaValuesOutput = z.output<
  typeof HooksSettingsSchema
>;
