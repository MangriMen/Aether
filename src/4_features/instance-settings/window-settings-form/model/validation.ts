import { z } from 'zod';

export const ResolutionPartSchema = z
  .string()
  .trim()
  .min(1, 'instanceSettings.windowError.resolution.emptyValue' as const)
  .regex(
    /^\d+$/,
    'instanceSettings.windowError.resolution.invalidValue' as const,
  )
  .transform(Number)
  .refine(
    (v) => v >= 1 && v <= 65535,
    'instanceSettings.windowError.resolution.invalidRange' as const,
  );

export const ResolutionSchema = z.object({
  width: ResolutionPartSchema,
  height: ResolutionPartSchema,
});

export const WindowSchema = z.object({
  resolution: ResolutionSchema,
});

export type WindowSettingsSchemaInput = z.input<typeof WindowSchema>;
export type WindowSettingsSchemaOutput = z.output<typeof WindowSchema>;
