export const javaKeys = {
  all: ['java'] as const,
  list: () => [...javaKeys.all, 'list'] as const,
  getActiveInstallations: () =>
    [...javaKeys.all, 'getActiveInstallations'] as const,
} as const;
