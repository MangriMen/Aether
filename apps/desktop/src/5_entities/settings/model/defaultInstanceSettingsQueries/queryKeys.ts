export const defaultInstanceSettingsKeys = {
  all: ['default_instance_settings'] as const,
  get: () => [...defaultInstanceSettingsKeys.all, 'get'] as const,
} as const;
