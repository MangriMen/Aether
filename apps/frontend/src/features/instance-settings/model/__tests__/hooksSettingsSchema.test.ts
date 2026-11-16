import * as v from 'valibot';
import { describe, expect, it } from 'vitest';

import {
  HOOK_COMMAND_ERRORS,
  HooksSettingsSchema,
} from '../hooksSettingsSchema';

const base = { preLaunch: '', wrapper: '', postExit: '' };

const errorsOf = (key: keyof typeof base, command: string) => {
  const result = v.safeParse(HooksSettingsSchema, { ...base, [key]: command });

  return result.issues?.map((issue) => issue.message);
};

describe('HooksSettingsSchema', () => {
  it('should_trim_the_edges_of_every_command', () => {
    const result = v.safeParse(HooksSettingsSchema, {
      preLaunch: '  a b ',
      wrapper: '\tprime-run  ',
      postExit: ' "x y" ',
    });

    expect(result.success).toBe(true);
    expect(result.output).toMatchObject({
      preLaunch: 'a b',
      wrapper: 'prime-run',
      postExit: '"x y"',
    });
  });

  it('should_accept_an_empty_command_when_the_hook_is_not_set', () => {
    expect(v.safeParse(HooksSettingsSchema, base).success).toBe(true);
    expect(
      v.safeParse(HooksSettingsSchema, { ...base, wrapper: '   ' }).success,
    ).toBe(true);
  });

  it.each(['preLaunch', 'wrapper', 'postExit'] as const)(
    'should_report_an_unclosed_quote_when_the_%s_command_has_one',
    (key) => {
      expect(errorsOf(key, 'echo "a b')).toEqual([
        HOOK_COMMAND_ERRORS.UNCLOSED_QUOTE,
      ]);
    },
  );

  it.each(['""', ' "" arg '])(
    'should_report_an_empty_program_when_the_command_is_%j',
    (command) => {
      expect(errorsOf('wrapper', command)).toEqual([HOOK_COMMAND_ERRORS.EMPTY]);
    },
  );
});
