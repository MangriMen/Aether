import * as v from 'valibot';

import { MAX_USERNAME_LENGTH, MIN_USERNAME_LENGTH } from './constants';

export type CreateOfflineAccountFormValues = {
  username: string;
};

const UsernameSchema = v.pipe(
  v.string(),
  v.minLength(MIN_USERNAME_LENGTH, 'INVALID_USERNAME_LENGTH'),
  v.maxLength(MAX_USERNAME_LENGTH, 'INVALID_USERNAME_LENGTH'),
  v.regex(/^\w+$/, 'INVALID_USERNAME_CHARS'),
);

export const CreateOfflineAccountFormSchema = v.object({
  username: UsernameSchema,
});

export type CreateOfflineAccountFormSchemaError =
  'INVALID_USERNAME_LENGTH' | 'INVALID_USERNAME_CHARS';

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
