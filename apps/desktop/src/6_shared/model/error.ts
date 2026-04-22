import type { FrontendError } from '../api';
import type { TFunction } from './i18nContext';

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

type LauncherErrorTranslateKey<E extends LauncherError> =
  E extends LauncherErrorWithoutGenericErrors
    ? `${E['type']}.${E['payload']['code']}`
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

export const getTranslatedError = (
  error: LauncherError,
  t: TFunction,
): string => {
  if (error.type === 'internal' || error.type === 'generic') {
    return error.payload;
  }

  const { key, args } = getLauncherErrorTranslationData(error);

  return t(`errors.${key}`, args) ?? '';
};
