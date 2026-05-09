import { z } from 'zod';

import type {
  ModdedLoaderVersion,
  Version,
  VersionType,
  ModLoader,
} from '@/entities/minecraft';

import { MOD_LOADERS, VERSION_TYPES } from '@/entities/minecraft';

export const ModLoaderSchema: z.ZodType<ModLoader> = z.enum(
  MOD_LOADERS as [ModLoader, ...ModLoader[]],
  {
    required_error: 'Mod loader is required',
  },
);

export const VersionSchema: z.ZodType<Version> = z.object({
  id: z.string().min(1),
  type: z.enum(VERSION_TYPES as [VersionType, ...VersionType[]]),
  url: z.string().min(1),
  time: z.string().min(1),
  releaseTime: z.string().min(1),
  sha1: z.string().min(1),
  complianceLevel: z.number(),
  originalSha1: z.string().min(1).optional(),
});

export const LoaderVersionSchema: z.ZodType<ModdedLoaderVersion> = z.object({
  id: z.string().min(0),
  url: z.string().min(0),
  stable: z.boolean(),
});

export const CreateCustomInstanceSchema = z
  .object({
    name: z
      .string({ required_error: 'Name is required' })
      .min(1, { message: 'Name is required' })
      .max(64, { message: 'Max length 64 symbols' }),
    gameVersion: z
      .string({ required_error: 'Game version is required' })
      .min(1, { message: 'Game version is required' }),
    loader: ModLoaderSchema,
    loaderVersionType: z.string().optional(),
    loaderVersion: z.string().optional(),
  })
  .superRefine((data, ctx) => {
    if (data.loaderVersionType === 'other' && !data.loaderVersion) {
      ctx.addIssue({
        code: 'custom',
        message: 'Loader version is required if loader version type is other',
        path: ['loaderVersion'],
      });
    }
  });

export const ImportInstanceSchema = z.object({
  pluginId: z.string().min(1),
  pathOrUrl: z.string().min(1, {
    message: 'Path is required',
  }),
});

export type ImportInstanceValues = z.infer<typeof ImportInstanceSchema>;
