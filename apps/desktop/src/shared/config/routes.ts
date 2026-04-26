import type { SettingsTab } from '@/pages/settings';

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
    tab ? `/settings/${encodeURIComponent(tab)}` : '/settings',

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
  SETTINGS: '/settings/:tab?',
  PLAYGROUND: '/playground',
} as const;
