export const CONTENT_ROUTES = {
  CONTENT: '/content',

  CONTENT_ITEM: (id: string) => `/content/${id}`,
  CONTENT_VERSION: (contentId: string, versionId: string) =>
    `/content/${contentId}/version/${versionId}`,
} as const;
