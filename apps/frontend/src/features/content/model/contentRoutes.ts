export const CONTENT_ROUTES = {
  CONTENT: '/content',

  CONTENT_ITEM: (id: string) => `/content/${id}`,
  CONTENT_ITEM_VERSIONS: (id: string) => `/content/${id}/versions`,
  CONTENT_VERSION: (contentId: string, versionId: string) =>
    `/content/${contentId}/versions/${versionId}`,
} as const;
