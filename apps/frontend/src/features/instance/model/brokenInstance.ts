import type { BrokenInstanceDto } from '@/platform/bindings/instance';
import type { TFunction } from '@/shared/i18n';

/** An instance the launcher stores but cannot read. */
export type BrokenInstance = BrokenInstanceDto;

export type BrokenInstanceReason = BrokenInstance['reason'];

/** Why an instance cannot be read, in words for the user. */
export const getBrokenReasonText = (
  reason: BrokenInstanceReason,
  t: TFunction,
): string => {
  const key = `instance.broken.reasons.${reason.code}` as const;
  const args = 'payload' in reason ? reason.payload : undefined;

  return t(key, args) ?? key;
};
