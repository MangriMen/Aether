export const updateKeys = {
  all: ['update'] as const,
  check: () => [...updateKeys.all, 'check'] as const,
};
