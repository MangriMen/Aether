export const MIN_JRE_MEMORY = 512;

export const MEMORY_SLIDER_HANDLE_DEBOUNCE = 300;

export const EnvVarsError = {
  InvalidFormat: 'invalidFormat',
  EmptyKey: 'emptyKey',
} as const;

export type EnvVarsError = (typeof EnvVarsError)[keyof typeof EnvVarsError];

export const ENV_VAR_ERRORS = Object.values(EnvVarsError);

export const isEnvVarsError = (error: unknown): error is EnvVarsError =>
  typeof error === 'string' && ENV_VAR_ERRORS.includes(error as EnvVarsError);
