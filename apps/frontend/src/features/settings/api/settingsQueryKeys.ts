export const settingsKeys = {
  all: ['settings'] as const,
  get: () => [...settingsKeys.all, 'get'] as const,
  storageUsage: () => [...settingsKeys.all, 'storageUsage'] as const,
  cacheUsage: () => [...settingsKeys.all, 'cacheUsage'] as const,
  cacheBusy: () => [...settingsKeys.all, 'cacheBusy'] as const,
  maxRam: () => [...settingsKeys.all, 'maxRam'] as const,
} as const;
