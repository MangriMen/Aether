import * as v from 'valibot';

import type { VersionType, ModLoader } from '@/entities/minecraft';

import { MOD_LOADERS, VERSION_TYPES } from '@/entities/minecraft';

export const ModLoaderSchema = v.picklist(
  MOD_LOADERS as [ModLoader, ...ModLoader[]],
  'Mod loader is required',
);

export const VersionSchema = v.object({
  id: v.pipe(v.string(), v.minLength(1)),
  type: v.picklist(VERSION_TYPES as [VersionType, ...VersionType[]]),
  url: v.pipe(v.string(), v.minLength(1)),
  time: v.pipe(v.string(), v.minLength(1)),
  releaseTime: v.pipe(v.string(), v.minLength(1)),
  sha1: v.pipe(v.string(), v.minLength(1)),
  complianceLevel: v.number(),
  originalSha1: v.optional(v.pipe(v.string(), v.minLength(1))),
});

export const LoaderVersionSchema = v.object({
  id: v.pipe(v.string(), v.minLength(0)),
  url: v.pipe(v.string(), v.minLength(0)),
  stable: v.boolean(),
});

export const CreateCustomInstanceSchema = v.pipe(
  v.object({
    name: v.pipe(
      v.string('Name is required'),
      v.minLength(1, 'Name is required'),
      v.maxLength(64, 'Max length 64 symbols'),
    ),
    gameVersion: v.pipe(
      v.string('Game version is required'),
      v.minLength(1, 'Game version is required'),
    ),
    loader: ModLoaderSchema,
    loaderVersionType: v.optional(v.string()),
    loaderVersion: v.optional(v.string()),
  }),
  v.forward(
    v.check(
      (data) => !(data.loaderVersionType === 'other' && !data.loaderVersion),
      'Loader version is required if loader version type is other',
    ),
    ['loaderVersion'],
  ),
);

export const ImportInstanceSchema = v.object({
  pluginId: v.pipe(v.string(), v.minLength(1)),
  pathOrUrl: v.pipe(v.string(), v.minLength(1, 'Path is required')),
});

export type CreateCustomInstanceSchemaOutput = v.InferOutput<
  typeof CreateCustomInstanceSchema
>;

export type ImportInstanceValues = v.InferOutput<typeof ImportInstanceSchema>;
