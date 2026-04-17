export const settingsKeys = {
  all: ['settings'] as const,
  get: () => [...settingsKeys.all, 'get'] as const,
  ram: () => [...settingsKeys.all, 'ram'] as const,
} as const;
