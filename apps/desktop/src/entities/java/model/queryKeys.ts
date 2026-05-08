export const javaKeys = {
  all: ['java'] as const,
  list: () => [...javaKeys.all, 'list'] as const,
} as const;
