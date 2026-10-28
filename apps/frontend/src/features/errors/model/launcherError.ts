import type { FrontendError } from '@/platform/bindings/application';
import type { TFunction } from '@/shared/i18n';

export type LauncherError = FrontendError;

export function isLauncherError(error: unknown): error is FrontendError {
  return (
    error !== null &&
    typeof error === 'object' &&
    'type' in error &&
    'payload' in error &&
    typeof error.type === 'string'
  );
}

type LauncherErrorWithoutGenericErrors = Exclude<
  LauncherError,
  { type: 'generic' | 'internal' }
>;

// The I/O message is picked by its kind in `getTranslatedError`, so it has no single key.
type LauncherErrorTranslateKey<E extends LauncherError> =
  E extends LauncherErrorWithoutGenericErrors
    ? Exclude<`${E['type']}.${E['payload']['code']}`, 'application.IO'>
    : never;

type LauncherErrorArgs<E extends LauncherError> =
  E extends LauncherErrorWithoutGenericErrors
    ? E['payload'] extends { payload: infer P }
      ? P
      : Record<string, never>
    : never;

const getLauncherErrorTranslationData = <
  E extends LauncherErrorWithoutGenericErrors,
>(
  error: E,
): {
  key: LauncherErrorTranslateKey<E>;
  args: LauncherErrorArgs<E> | undefined;
} => {
  const key = `${error.type}.${error.payload.code}`;
  const args = 'payload' in error.payload ? error.payload.payload : undefined;

  return {
    key: key as LauncherErrorTranslateKey<E>,
    args: args as LauncherErrorArgs<E>,
  };
};

type IoErrorKind = 'notFound' | 'permissionDenied' | 'other';

const getIoErrorKind = (payload: unknown): IoErrorKind => {
  if (
    payload !== null &&
    typeof payload === 'object' &&
    'kind' in payload &&
    (payload.kind === 'notFound' || payload.kind === 'permissionDenied')
  ) {
    return payload.kind;
  }

  return 'other';
};

export const getTranslatedError = (
  error: LauncherError,
  t: TFunction,
): string => {
  if (error.type === 'internal' || error.type === 'generic') {
    return error.payload;
  }

  if (error.type === 'application' && error.payload.code === 'IO') {
    const key =
      `errors.application.IO.${getIoErrorKind(error.payload.payload)}` as const;

    return t(key, error.payload.payload) ?? key;
  }

  const { key, args } = getLauncherErrorTranslationData(error);

  return t(`errors.${key}`, args) ?? key;
};
