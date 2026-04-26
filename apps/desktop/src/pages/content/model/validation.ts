import z from 'zod';

import { ContentType } from '../../../entities/instances';

const ContentTypeSchema = z.nativeEnum(ContentType);

const ProviderIdSchema = z.object({
  pluginId: z.string(),
  capabilityId: z.string(),
});

export const ContentSearchSchema = z.object({
  page: z.number().min(1),
  pageSize: z.number().min(1),
  query: z.string(),
  providerId: ProviderIdSchema,
  contentType: ContentTypeSchema,
  gameVersions: z.string().array(),
  loaders: z.string().array(),
});

export type ContentSearchInputValues = z.input<typeof ContentSearchSchema>;
export type ContentSearchOutputValues = z.output<typeof ContentSearchSchema>;
