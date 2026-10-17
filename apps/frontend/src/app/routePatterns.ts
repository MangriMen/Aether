import { APP_ROUTES } from '@/app/shell';
import { CONTENT_ROUTES } from '@/features/content';

// Patterns for Route definition. Static paths come from the route builders,
// so a page and the links to it cannot drift apart.
export const ROUTE_PATTERNS = {
  HOME: APP_ROUTES.HOME,
  CONTENT: CONTENT_ROUTES.CONTENT,
  CONTENT_ITEM_REL: '/:contentId',
  INSTANCE: '/instances/:id',
  INSTANCE_SETTINGS_REL: 'settings', // Relative path
  INSTANCE_DIALOG: '/instance-settings/:id',
  PLAYGROUND: APP_ROUTES.PLAYGROUND,
} as const;
