export const settingsKeys = {
  all: ['settings'] as const,
  get: () => [...settingsKeys.all, 'get'] as const,
  maxRam: () => [...settingsKeys.all, 'maxRam'] as const,
} as const;
