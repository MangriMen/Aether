export const appSettingsKeys = {
  all: ['app_settings'] as const,
  get: () => [...appSettingsKeys.all, 'get'] as const,
} as const;
