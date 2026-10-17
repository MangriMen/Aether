// Patterns for Route definition
export const ROUTE_PATTERNS = {
  HOME: '/',
  CONTENT: '/content',
  CONTENT_ITEM_REL: '/:contentId',
  INSTANCE: '/instances/:id',
  INSTANCE_SETTINGS_REL: 'settings', // Relative path
  INSTANCE_DIALOG: '/instance-settings/:id',
  PLAYGROUND: '/playground',
} as const;
