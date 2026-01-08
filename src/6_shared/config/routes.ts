import type { SettingsTab } from '@/pages/settings/model/settingsTabs';

export const ROUTES = {
  HOME: '/',

  CONTENT: '/content',

  INSTANCE: (id: string | number) => `/instances/${encodeURIComponent(id)}`,

  INSTANCE_SETTINGS: (id: string | number) =>
    `/instances/${encodeURIComponent(id)}/settings`,

  INSTANCE_DIALOG: (id: string | number) =>
    `/instance-settings/${encodeURIComponent(id)}`,

  SETTINGS: (tab?: SettingsTab) =>
    tab ? `/settings/${encodeURIComponent(tab)}` : '/settings',
};

// Patterns for Route definition
export const ROUTE_PATTERNS = {
  HOME: '/',
  CONTENT: '/content',
  INSTANCE: '/instances/:id',
  INSTANCE_SETTINGS: 'settings', // Relative path
  INSTANCE_DIALOG: '/instance-settings/:id',
  SETTINGS: '/settings/:tab?',
};
