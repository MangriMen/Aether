export const defaultInstanceSettingsQueryKeys = {
  all: ['default_instance_settings'] as const,
  get: () => [...defaultInstanceSettingsQueryKeys.all, 'get'] as const,
} as const;
