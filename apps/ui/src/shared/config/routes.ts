import type { SettingsTab } from '@/widgets/settings-view';

export const ROUTES = {
  HOME: '/',

  CONTENT: '/content',

  CONTENT_ITEM: (id: string) => `/content/${id}`,

  INSTANCE: (id: string | number) => `/instances/${encodeURIComponent(id)}`,

  INSTANCE_SETTINGS: (id: string | number) =>
    `/instances/${encodeURIComponent(id)}/settings`,

  INSTANCE_DIALOG: (id: string | number) =>
    `/instance-settings/${encodeURIComponent(id)}`,

  SETTINGS: (tab?: SettingsTab) =>
    tab ? `?modal=settings&tab=${encodeURIComponent(tab)}` : '?modal=settings',

  PLAYGROUND: '/playground',
} as const;

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
