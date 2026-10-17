import { makePersisted } from '@solid-primitives/storage';
import { createSignal } from 'solid-js';

export const UPDATE_NOTIFICATION_STYLE_KEY = 'aether:update-notification-style';

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
