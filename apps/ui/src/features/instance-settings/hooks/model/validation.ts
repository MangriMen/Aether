import { z } from 'zod';

export const HooksSettingsSchema = z.object({
  preLaunch: z
    .string()
    .nullable()
    .transform((value) => (value === '' ? null : value)),
  wrapper: z
    .string()
    .nullable()
    .transform((value) => (value === '' ? null : value)),
  postExit: z
    .string()
    .nullable()
    .transform((value) => (value === '' ? null : value)),
});

export type HooksSettingsSchemaInput = z.input<typeof HooksSettingsSchema>;
export type HooksSettingsSchemaOutput = z.output<typeof HooksSettingsSchema>;
