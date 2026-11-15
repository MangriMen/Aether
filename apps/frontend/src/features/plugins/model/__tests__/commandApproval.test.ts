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
      options: ['-Xmx1G'],
      target: '#/cache/a.jar',
      rest: ['--url', 'https://x'],
    });
  });

  it('should_show_everything_as_arguments_when_it_is_not_a_jar_launch', () => {
    expect(splitCommandLine(['-cp', 'lib.jar', 'com.Main'])).toEqual({
      options: [],
      target: undefined,
      rest: ['-cp', 'lib.jar', 'com.Main'],
    });
  });

  it('should_not_mistake_the_value_of_an_option_for_the_program', () => {
    expect(
      splitCommandLine(['--add-opens', 'java.base/x=ALL', '-jar', 'a.jar']),
    ).toEqual({
      options: ['--add-opens', 'java.base/x=ALL'],
      target: 'a.jar',
      rest: [],
    });
  });

  it('should_leave_the_target_empty_when_the_jar_flag_ends_the_command', () => {
    expect(splitCommandLine(['-Xmx1G', '-jar'])).toEqual({
      options: [],
      target: undefined,
      rest: ['-Xmx1G', '-jar'],
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
