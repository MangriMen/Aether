const ERROR_KINDS = ['authError', 'instanceError', 'settingsError'] as const;
type ErrorKind = (typeof ERROR_KINDS)[number];

type ErrorCode =
  | `${ErrorKind}.${string}`
  | `${ErrorKind}.${string}.${string}`
  | `${ErrorKind}.${string}.${string}.${string}`
  | `${ErrorKind}.${string}.${string}.${string}.${string}`;

interface BaseError<Code extends ErrorCode, Fields> {
  code: Code;
  fields: Fields;
  message: string;
}

export type LauncherError =
  | BaseError<'authError.credentialsNotFound', { id: string }>
  | BaseError<'authError.noActiveCredentials', null>
  | BaseError<'authError.storageFailure', null>
  | BaseError<
      'instanceError.credentialsError.credentialsNotFound',
      { id: string }
    >
  | BaseError<'instanceError.credentialsError.noActiveCredentials', null>
  | BaseError<'instanceError.credentialsError.storageFailure', null>
  | BaseError<'instanceError.storageFailure', null>
  | BaseError<'settingsError.storageFailure', null>;

export const isLauncherError = (error: unknown): error is LauncherError => {
  return (
    !!error &&
    typeof error === 'object' &&
    'code' in error &&
    'fields' in error &&
    'message' in error
  );
};

export const isErrorKind = (value: string): value is ErrorKind =>
  ERROR_KINDS.includes(value as ErrorKind);
