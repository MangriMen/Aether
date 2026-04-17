export const appSettingsQueryKeys = {
  all: ['app_settings'] as const,
  get: () => [...appSettingsQueryKeys.all, 'get'] as const,
} as const;
