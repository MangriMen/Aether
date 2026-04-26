export const processKeys = {
  all: ['process'] as const,
  list: () => [...processKeys.all, 'list'] as const,
  byInstance: (id: string) => [...processKeys.all, 'instance', id] as const,
} as const;
