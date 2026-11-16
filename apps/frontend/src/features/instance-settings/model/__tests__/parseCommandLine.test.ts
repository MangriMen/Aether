import { describe, expect, it } from 'vitest';

import fixture from '../../../../../../../packages/aether-core/tests/fixtures/command-line.json';
import { parseCommandLine } from '../parseCommandLine';

type Vector = {
  name: string;
  input: string;
  program?: string;
  args?: string[];
  error?: string;
};

describe('parseCommandLine', () => {
  it.each(fixture.vectors as Vector[])(
    'should_match_the_core_parser_when_$name',
    ({ input, program, args, error }) => {
      expect(parseCommandLine(input)).toEqual(
        error ? { error } : { program, args },
      );
    },
  );
});
