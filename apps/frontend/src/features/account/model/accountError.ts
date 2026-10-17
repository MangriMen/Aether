import type { AuthErrorDto } from '@/platform/bindings/auth';

export const isAuthValidationError = (error: AuthErrorDto) =>
  error.code === 'INVALID_USERNAME_CHARS' ||
  error.code === 'INVALID_USERNAME_LENGTH';
