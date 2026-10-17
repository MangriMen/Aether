import * as v from 'valibot';

const ResolutionPartSchema = v.pipe(
  v.string(),
  v.trim(),
  v.minLength(1, 'instanceSettings.windowError.resolution.emptyValue'),
  v.regex(/^\d+$/, 'instanceSettings.windowError.resolution.invalidValue'),
  v.transform(Number),
  v.number(),
  v.minValue(1, 'instanceSettings.windowError.resolution.invalidRange'),
  v.maxValue(65535, 'instanceSettings.windowError.resolution.invalidRange'),
);

const ResolutionSchema = v.object({
  width: ResolutionPartSchema,
  height: ResolutionPartSchema,
});

export const WindowSettingsSchema = v.object({
  resolution: ResolutionSchema,
  forceFullscreen: v.optional(v.boolean()),
  overrideWindowSettings: v.optional(v.boolean()),
});

export type WindowSettingsSchemaInput = v.InferInput<
  typeof WindowSettingsSchema
>;
export type WindowSettingsSchemaOutput = v.InferOutput<
  typeof WindowSettingsSchema
>;
