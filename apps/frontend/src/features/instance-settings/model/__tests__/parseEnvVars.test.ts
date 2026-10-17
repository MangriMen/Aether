import { describe, expect, it } from 'vitest';

import { envVarsToString, stringToEnvVars } from '../parseEnvVars';

describe('stringToEnvVars', () => {
  it('splits variables on the first =', () => {
    expect(stringToEnvVars('A=1; JAVA_TOOL_OPTIONS=-Dfoo=bar')).toEqual([
      ['A', '1'],
      ['JAVA_TOOL_OPTIONS', '-Dfoo=bar'],
    ]);
  });

  it('skips parts without a value', () => {
    expect(stringToEnvVars('A=1;B; = ;')).toEqual([['A', '1']]);
  });

  it('reads back what envVarsToString writes', () => {
    const vars: [string, string][] = [
      ['A', 'x=y'],
      ['B', ''],
    ];

    expect(stringToEnvVars(envVarsToString(vars) ?? '')).toEqual(vars);
  });
});
