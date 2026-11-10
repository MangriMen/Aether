import { describe, expect, it } from 'vitest';

import { logFilePath } from '../logPath';

describe('logFilePath', () => {
  it('should look under .metadata when the log is a captured console log', () => {
    expect(
      logFilePath(String.raw`C:\inst\a`, {
        kind: 'console',
        name: 'console-1.log',
      }),
    ).toBe(String.raw`C:\inst\a\.metadata\logs\console-1.log`);
  });

  it('should look under logs when the log is a game log, with the separator of the path', () => {
    expect(
      logFilePath('/home/inst', { kind: 'game', name: 'latest.log' }),
    ).toBe('/home/inst/logs/latest.log');
  });
});
