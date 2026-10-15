import { makePersisted } from '@solid-primitives/storage';
import { createSignal } from 'solid-js';

export const IS_DEBUG_KEY = 'aether:debug';
export const IS_DEVELOPER_MODE_KEY = 'aether:developer-mode';
export const UPDATE_NOTIFICATION_STYLE_KEY = 'aether:update-notification-style';

export const [isDebug, setIsDebug] = makePersisted(createSignal(false), {
  name: IS_DEBUG_KEY,
});

export const [isDeveloperMode, setIsDeveloperMode] = makePersisted(
  createSignal(false),
  { name: IS_DEVELOPER_MODE_KEY },
);

export const UpdateNotificationStyle = {
  Toast: 'toast',
  Banner: 'banner',
} as const;

export const UpdateNotificationStyles = Object.values(UpdateNotificationStyle);

export type UpdateNotificationStyle =
  (typeof UpdateNotificationStyle)[keyof typeof UpdateNotificationStyle];

export const [updateNotificationStyle, setUpdateNotificationStyle] =
  makePersisted(
    createSignal<UpdateNotificationStyle>(UpdateNotificationStyle.Toast),
    { name: UPDATE_NOTIFICATION_STYLE_KEY },
  );
