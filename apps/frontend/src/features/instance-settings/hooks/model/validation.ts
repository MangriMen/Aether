import { z } from 'zod';

export const HooksSettingsSchema = z.object({
  preLaunch: z.string(),
  wrapper: z.string(),
  postExit: z.string(),
  overrideHooks: z.boolean().optional(),
});

export type HooksSettingsSchemaInput = z.input<typeof HooksSettingsSchema>;
export type HooksSettingsSchemaOutput = z.output<typeof HooksSettingsSchema>;
