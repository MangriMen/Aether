import { describe, expect, it } from 'vitest';

import { byExpiry, secondsLeft, splitCommandLine } from '../commandApproval';

describe('splitCommandLine', () => {
  it('should_separate_the_options_the_target_and_the_arguments', () => {
    expect(
      splitCommandLine([
        '-Xmx1G',
        '-jar',
        '#/cache/a.jar',
        '--url',
        'https://x',
      ]),
    ).toEqual({
      options: ['-Xmx1G', '-jar'],
      target: '#/cache/a.jar',
      rest: ['--url', 'https://x'],
    });
  });

  it('should_leave_the_target_empty_when_every_token_is_an_option', () => {
    expect(splitCommandLine(['-version'])).toEqual({
      options: ['-version'],
      target: undefined,
      rest: [],
    });
  });
});

describe('secondsLeft', () => {
  it('should_round_up_and_stop_at_zero', () => {
    const expires = '2026-11-16T10:00:10.500Z';

    expect(secondsLeft(expires, Date.parse('2026-11-16T10:00:00Z'))).toBe(11);
    expect(secondsLeft(expires, Date.parse('2026-11-16T10:01:00Z'))).toBe(0);
    expect(secondsLeft('nonsense', 0)).toBe(0);
  });
});

describe('byExpiry', () => {
  it('should_put_the_oldest_question_first', () => {
    const sorted = byExpiry([
      { id: 'b', expires_at: '2026-11-16T10:00:20Z' },
      { id: 'a', expires_at: '2026-11-16T10:00:10Z' },
    ]);

    expect(sorted.map((x) => x.id)).toEqual(['a', 'b']);
  });
});
