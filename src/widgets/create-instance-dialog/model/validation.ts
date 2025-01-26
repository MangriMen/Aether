import { z } from 'zod';

import type { LoaderVersion, Version } from '@/entities/minecraft';
import { ModLoader, VersionType } from '@/entities/minecraft';

export const ModLoaderSchema: z.ZodType<ModLoader> = z.enum(
  [
    ModLoader.Vanilla,
    ModLoader.Fabric,
    ModLoader.Forge,
    ModLoader.Quilt,
    ModLoader.NeoForge,
  ],
  { required_error: 'Mod loader is required' },
);

export const VersionSchema: z.ZodType<Version> = z.object({
  id: z.string().min(1),
  type: z.enum([
    VersionType.Release,
    VersionType.Snapshot,
    VersionType.OldAlpha,
    VersionType.OldBeta,
  ]),
  url: z.string().min(1),
  time: z.string().min(1),
  release_time: z.string().min(1),
  sha1: z.string().min(1),
  compliance_level: z.number(),
  original_sha1: z.string().min(1).optional(),
});

export const LoaderVersionSchema: z.ZodType<LoaderVersion> = z.object({
  id: z.string().min(0),
  url: z.string().min(0),
  stable: z.boolean(),
});

export const CreateCustomInstanceSchema = z
  .object({
    name: z
      .string({ required_error: 'Name is required' })
      .min(1, { message: 'Name is required' }),
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
