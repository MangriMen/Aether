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

export type HooksSettingsSchemaValuesInput = z.input<
  typeof HooksSettingsSchema
>;
export type HooksSettingsSchemaValuesOutput = z.output<
  typeof HooksSettingsSchema
>;
