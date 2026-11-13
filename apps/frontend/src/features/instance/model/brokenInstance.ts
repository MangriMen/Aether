import type { BrokenInstanceDto } from '@/platform/bindings/instance';
import type { TFunction } from '@/shared/i18n';

import { getCauseText } from '@/features/errors';

/** An instance the launcher stores but cannot read. */
export type BrokenInstance = BrokenInstanceDto;

export type BrokenInstanceReason = BrokenInstance['reason'];

/** Why an instance cannot be read, in words for the user. */
export const getBrokenReasonText = (
  reason: BrokenInstanceReason,
  t: TFunction,
): string => {
  const key = `instance.broken.reasons.${reason.code}` as const;

  // The cause is a category of its own, translated apart from the reason.
  if (reason.code === 'UNREADABLE_FILE') {
    const text = t(key) ?? key;
    const causeText = getCauseText(reason.payload.cause, t);

    return causeText ? `${text}: ${causeText}` : text;
  }

  const args = 'payload' in reason ? reason.payload : undefined;

  return t(key, args) ?? key;
};
