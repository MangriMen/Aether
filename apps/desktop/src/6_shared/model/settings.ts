import { makePersisted } from '@solid-primitives/storage';
import { createSignal } from 'solid-js';

export const IS_DEBUG_KEY = '_AETHER_DEBUG';
export const IS_DEVELOPER_MODE_KEY = '_AETHER_DEVELOPER_MODE';
export const UPDATE_NOTIFICATION_STYLE_KEY =
  '_AETHER_UPDATE_NOTIFICATION_STYLE';

export const [isDebug, setIsDebug] = makePersisted(
  // eslint-disable-next-line solid/reactivity
  createSignal(false),
  {
    name: IS_DEBUG_KEY,
  },
);

export const [isDeveloperMode, setIsDeveloperMode] = makePersisted(
  // eslint-disable-next-line solid/reactivity
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
    // eslint-disable-next-line solid/reactivity
    createSignal<UpdateNotificationStyle>(UpdateNotificationStyle.Toast),
    { name: UPDATE_NOTIFICATION_STYLE_KEY },
  );
