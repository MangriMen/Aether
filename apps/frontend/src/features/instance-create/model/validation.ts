import * as v from 'valibot';

import type { ModLoader } from '@/features/minecraft';

import { MOD_LOADERS } from '@/features/minecraft';

// Messages are dictionary keys: the form translates them, see `isValidationKey`.
const VALIDATION_KEYS = [
  'validation.nameRequired',
  'validation.nameTooLong',
  'validation.gameVersionRequired',
  'validation.loaderRequired',
  'validation.loaderVersionRequired',
] as const;

export type ValidationKey = (typeof VALIDATION_KEYS)[number];

export const isValidationKey = (message: string): message is ValidationKey =>
  (VALIDATION_KEYS as readonly string[]).includes(message);

const ModLoaderSchema = v.picklist(
  MOD_LOADERS as [ModLoader, ...ModLoader[]],
  'validation.loaderRequired',
);

export const CreateCustomInstanceSchema = v.pipe(
  v.object({
    name: v.pipe(
      v.string('validation.nameRequired'),
      v.minLength(1, 'validation.nameRequired'),
      v.maxLength(64, 'validation.nameTooLong'),
    ),
    gameVersion: v.pipe(
      v.string('validation.gameVersionRequired'),
      v.minLength(1, 'validation.gameVersionRequired'),
    ),
    loader: ModLoaderSchema,
    loaderVersionType: v.optional(v.string()),
    loaderVersion: v.optional(v.string()),
  }),
  v.forward(
    v.check(
      // Vanilla has no loader version, whatever the hidden advanced section holds.
      (data) =>
        data.loader === 'vanilla' ||
        !(data.loaderVersionType === 'other' && !data.loaderVersion),
      'validation.loaderVersionRequired',
    ),
    ['loaderVersion'],
  ),
);

export type CreateCustomInstanceSchemaOutput = v.InferOutput<
  typeof CreateCustomInstanceSchema
>;
