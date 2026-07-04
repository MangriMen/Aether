import * as v from 'valibot';

import { ContentType } from '@/entities/instances';

const ContentTypeSchema = v.enum(ContentType);

const ProviderIdSchema = v.object({
  pluginId: v.string(),
  capabilityId: v.string(),
});

export const ContentSearchSchema = v.object({
  page: v.pipe(v.number(), v.minValue(1)),
  pageSize: v.pipe(v.number(), v.minValue(1)),
  query: v.string(),
  providerId: ProviderIdSchema,
  contentType: ContentTypeSchema,
  gameVersions: v.array(v.string()),
  loaders: v.array(v.string()),
});

export type ContentSearchInputValues = v.InferInput<typeof ContentSearchSchema>;
export type ContentSearchOutputValues = v.InferOutput<
  typeof ContentSearchSchema
>;
