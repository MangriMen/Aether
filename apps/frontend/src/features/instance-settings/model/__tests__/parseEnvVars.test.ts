import { describe, expect, it } from 'vitest';

import { envVarsToString, stringToEnvVars } from '../parseEnvVars';

describe('stringToEnvVars', () => {
  it('splits variables on whitespace or ;', () => {
    expect(stringToEnvVars('A=1 B=2')).toEqual({
      vars: [
        ['A', '1'],
        ['B', '2'],
      ],
    });
    expect(stringToEnvVars('A=1; B=2;')).toEqual({
      vars: [
        ['A', '1'],
        ['B', '2'],
      ],
    });
  });

  it('keeps text without NAME= in the previous value', () => {
    expect(
      stringToEnvVars(
        'PATH=C:/jdk/bin;C:/tools JAVA_HOME=C:\\Program Files\\Java; A=',
      ),
    ).toEqual({
      vars: [
        ['PATH', 'C:/jdk/bin;C:/tools'],
        ['JAVA_HOME', 'C:\\Program Files\\Java'],
        ['A', ''],
      ],
    });
    expect(stringToEnvVars('JAVA_TOOL_OPTIONS=-Dfoo=bar -Dx=1')).toEqual({
      vars: [['JAVA_TOOL_OPTIONS', '-Dfoo=bar -Dx=1']],
    });
  });

  it('takes a quoted value as is', () => {
    expect(stringToEnvVars('X="a B=c; " Y="say ""hi"""')).toEqual({
      vars: [
        ['X', 'a B=c; '],
        ['Y', 'say "hi"'],
      ],
    });
    expect(stringToEnvVars('X="-Da=b c" -Dd=e')).toEqual({
      vars: [['X', '"-Da=b c" -Dd=e']],
    });
    expect(stringToEnvVars('X="a B=c" d E=f')).toEqual({
      vars: [
        ['X', '"a B=c" d'],
        ['E', 'f'],
      ],
    });
    expect(stringToEnvVars('X="a; b')).toEqual({ vars: [['X', '"a; b']] });
  });

  it('reports text before the first variable', () => {
    expect(stringToEnvVars('B A=1')).toEqual({
      error: 'invalidFormat',
      part: 'B',
    });
    expect(stringToEnvVars(' =1')).toEqual({ error: 'emptyKey', part: '=1' });
    expect(stringToEnvVars('A-B=1')).toEqual({
      error: 'invalidKey',
      part: 'A-B=1',
    });
  });

  it('reads back what envVarsToString writes', () => {
    const vars: [string, string][] = [
      ['PATH', 'C:/jdk/bin;C:/tools'],
      ['A', 'x B=y'],
      ['B', ''],
      ['C', ' padded; '],
      ['D', '"quoted"'],
      ['E', 'ends with "'],
    ];

    expect(envVarsToString(vars)).toBe(
      'PATH=C:/jdk/bin;C:/tools; A="x B=y"; B=; C=" padded; "; D="""quoted"""; E=ends with "',
    );
    expect(stringToEnvVars(envVarsToString(vars) ?? '')).toEqual({ vars });
  });
});
