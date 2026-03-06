import z from 'zod';

import { ContentType } from '@/entities/instances';

const ContentTypeSchema = z.nativeEnum(ContentType);

export const ContentSearchSchema = z.object({
  page: z.number(),
  pageSize: z.number(),
  query: z.string(),
  provider: z
    .object({
      pluginId: z.string(),
      capabilityId: z.string(),
    })
    .transform((value) => `${value.pluginId}_${value.capabilityId}`),
  contentType: ContentTypeSchema,
  gameVersions: z.string().array(),
  loaders: z.string().array(),
});

export type ContentSearchInputValues = z.input<typeof ContentSearchSchema>;
export type ContentSearchOutputValues = z.output<typeof ContentSearchSchema>;
