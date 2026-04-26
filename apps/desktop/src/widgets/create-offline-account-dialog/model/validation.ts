import { z } from 'zod';

import { MAX_USERNAME_LENGTH, MIN_USERNAME_LENGTH } from './constants';

export type CreateOfflineAccountFormValues = {
  username: string;
};

export const CreateOfflineAccountFormSchema = z.object({
  username: z
    .string()
    // .min and .max are used for clearer error reporting than a single regex
    .min(MIN_USERNAME_LENGTH, {
      message: 'INVALID_USERNAME_LENGTH',
    })
    .max(MAX_USERNAME_LENGTH, {
      message: 'INVALID_USERNAME_LENGTH',
    })
    // Matches: ASCII alphanumeric or underscore
    // eslint-disable-next-line sonarjs/concise-regex
    .regex(/^[a-zA-Z0-9_]+$/, {
      message: 'INVALID_USERNAME_CHARS',
    }),
});

export type CreateOfflineAccountFormSchemaError =
  | 'INVALID_USERNAME_LENGTH'
  | 'INVALID_USERNAME_CHARS';

type EnsureAllKeys = { [K in CreateOfflineAccountFormSchemaError]: K };

export const CreateOfflineAccountFormSchemaErrors = Object.values({
  INVALID_USERNAME_LENGTH: 'INVALID_USERNAME_LENGTH',
  INVALID_USERNAME_CHARS: 'INVALID_USERNAME_CHARS',
} satisfies EnsureAllKeys);

export const isCreateOfflineAccountFormSchemaError = (
  error: unknown,
): error is CreateOfflineAccountFormSchemaError =>
  CreateOfflineAccountFormSchemaErrors.includes(
    error as CreateOfflineAccountFormSchemaError,
  );
