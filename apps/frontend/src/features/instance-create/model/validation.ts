import * as v from 'valibot';

import type { ModLoader } from '@/features/minecraft';

import { MOD_LOADERS } from '@/features/minecraft';

const ModLoaderSchema = v.picklist(
  MOD_LOADERS as [ModLoader, ...ModLoader[]],
  'Mod loader is required',
);

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

export type CreateCustomInstanceSchemaOutput = v.InferOutput<
  typeof CreateCustomInstanceSchema
>;
