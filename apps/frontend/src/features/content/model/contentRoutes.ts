export const CONTENT_ROUTES = {
  CONTENT: '/content',

  CONTENT_ITEM: (id: string) => `/content/${id}`,
} as const;
