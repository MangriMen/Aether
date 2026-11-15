import type {
  CommandAnswerDto,
  PendingCommandApprovalDto,
} from '@/platform/bindings/plugin';

export type CommandAnswer = CommandAnswerDto;
export type PendingCommandApproval = PendingCommandApprovalDto;

/** The command line of a waiting command, split for showing. */
export interface CommandLineParts {
  /** Everything before the program file: the JVM options. */
  options: string[];
  /** The program file after `-jar`, as the plugin wrote it. */
  target: string | undefined;
  /** What follows the target. Can hold a token (a pack URL), so it is shown folded. */
  rest: string[];
}

/**
 * A `java -jar` launch: what comes before `-jar` is JVM options, the token after it the program,
 * the rest its arguments. Anything else is not told apart (an option may take its value as a
 * separate token), so it is all shown as arguments.
 */
export const splitCommandLine = (args: readonly string[]): CommandLineParts => {
  const jarIndex = args.indexOf('-jar');
  const target = jarIndex === -1 ? undefined : args[jarIndex + 1];

  if (target === undefined) {
    return { options: [], target: undefined, rest: [...args] };
  }

  return {
    options: args.slice(0, jarIndex),
    target,
    rest: args.slice(jarIndex + 2),
  };
};

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
