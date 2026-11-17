import type {
  CommandAnswerDto,
  PendingCommandApprovalDto,
} from '@/platform/bindings/plugin';

export type CommandAnswer = CommandAnswerDto;
export type PendingCommandApproval = PendingCommandApprovalDto;

/** Whole seconds until `expiresAt`, never below zero; zero when the date does not parse. */
export const secondsLeft = (expiresAt: string, now: number): number => {
  const expires = Date.parse(expiresAt);

  return Number.isNaN(expires)
    ? 0
    : Math.max(0, Math.ceil((expires - now) / 1000));
};

/** The questions oldest first, which is the order they are answered in. */
export const byExpiry = <T extends { expires_at: string }>(
  approvals: readonly T[],
): T[] =>
  [...approvals].sort(
    (a, b) => Date.parse(a.expires_at) - Date.parse(b.expires_at),
  );
