import { makePersisted } from '@solid-primitives/storage';
import { createSignal } from 'solid-js';
import * as v from 'valibot';

const UPDATE_NOTIFICATION_STYLE_KEY = 'aether:update-notification-style';

export const UpdateNotificationStyle = {
  Toast: 'toast',
  Banner: 'banner',
} as const;

export type UpdateNotificationStyle =
  (typeof UpdateNotificationStyle)[keyof typeof UpdateNotificationStyle];

export const deserializeUpdateNotificationStyle = (
  raw: string,
): UpdateNotificationStyle => {
  try {
    const stored: unknown = JSON.parse(raw);

    if (v.is(v.enum(UpdateNotificationStyle), stored)) {
      return stored;
    }
  } catch {
    // not JSON
  }

  return UpdateNotificationStyle.Toast;
};

export const [updateNotificationStyle, setUpdateNotificationStyle] =
  makePersisted(
    createSignal<UpdateNotificationStyle>(UpdateNotificationStyle.Toast),
    {
      name: UPDATE_NOTIFICATION_STYLE_KEY,
      deserialize: deserializeUpdateNotificationStyle,
    },
  );
