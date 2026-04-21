import type { FrontendError } from '../api';
import type { TFunction } from './i18nContext';

export type LauncherError = FrontendError;

export const isLauncherError = (error: unknown): error is LauncherError => {
  return (
    !!error &&
    typeof error === 'object' &&
    'code' in error &&
    'fields' in error &&
    'message' in error
  );
};

export const getTranslatedError = (
  error: LauncherError,
  t: TFunction,
): string => {
  if (error.type === 'internal' || error.type === 'generic') {
    return error.payload;
  }

  if (error.type === 'auth') {
    return t(`backendError.auth.${error.payload.code}`);
  }

  return t(
    `backendError.${error.type}.${error.payload.code}`,
    error.payload ?? undefined,
  );
};
