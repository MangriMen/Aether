import { describe, expect, it } from 'vitest';

import { envVarsToString, stringToEnvVars } from '../parseEnvVars';

describe('stringToEnvVars', () => {
  it('splits variables on the first =', () => {
    expect(stringToEnvVars('A=1; JAVA_TOOL_OPTIONS=-Dfoo=bar;')).toEqual({
      vars: [
        ['A', '1'],
        ['JAVA_TOOL_OPTIONS', '-Dfoo=bar'],
      ],
    });
  });

  it('reports a part without = or without a key', () => {
    expect(stringToEnvVars('A=1;B')).toEqual({
      error: 'invalidFormat',
      part: 'B',
    });
    expect(stringToEnvVars('A=1; = ')).toEqual({
      error: 'emptyKey',
      part: '=',
    });
  });

  it('reads back what envVarsToString writes', () => {
    const vars: [string, string][] = [
      ['A', 'x=y'],
      ['B', ''],
    ];

    expect(stringToEnvVars(envVarsToString(vars) ?? '')).toEqual({ vars });
  });
});
