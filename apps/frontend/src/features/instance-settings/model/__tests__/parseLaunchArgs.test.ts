import { describe, expect, it } from 'vitest';

import { launchArgsToString, stringToLaunchArgs } from '../parseLaunchArgs';

describe('stringToLaunchArgs', () => {
  it('splits on any whitespace and skips empty parts', () => {
    expect(stringToLaunchArgs('  -Xss1M   -XX:+UseG1GC ')).toEqual([
      '-Xss1M',
      '-XX:+UseG1GC',
    ]);
    expect(stringToLaunchArgs('')).toEqual([]);
  });

  it('keeps quoted spaces and drops the quotes', () => {
    expect(
      stringToLaunchArgs('-Dx="a b" "-Dpath=C:\\Program Files\\" ""'),
    ).toEqual(['-Dx=a b', '-Dpath=C:\\Program Files\\', '']);
    expect(stringToLaunchArgs('"say ""hi"""')).toEqual(['say "hi"']);
  });

  it('reads an unclosed quote as a plain character', () => {
    expect(stringToLaunchArgs('-Dx="a b -Xmx2G')).toEqual([
      '-Dx="a',
      'b',
      '-Xmx2G',
    ]);
    expect(stringToLaunchArgs('"a b" -Dy="c')).toEqual(['a b', '-Dy="c']);
  });

  it('reads back what launchArgsToString writes', () => {
    const args = ['-Xss1M', '-Dx=a b', '', '-Dq="v"', 'C:\\dir\\'];

    expect(launchArgsToString(args)).toBe(
      '-Xss1M "-Dx=a b" "" "-Dq=""v""" C:\\dir\\',
    );
    expect(stringToLaunchArgs(launchArgsToString(args) ?? '')).toEqual(args);
  });
});
