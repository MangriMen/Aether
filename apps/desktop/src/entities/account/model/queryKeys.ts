export const accountKeys = {
  all: ['account'] as const,
  list: () => [...accountKeys.all, 'list'] as const,
} as const;
