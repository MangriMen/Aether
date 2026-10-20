import { describe, expect, it } from 'vitest';

import { reconcileGameVersion } from '../gameVersionSelection';

describe('reconcileGameVersion', () => {
  const available = [{ id: '1.20.1' }, { id: '1.19.4' }];

  it('keeps a version that is still offered', () => {
    expect(reconcileGameVersion('1.20.1', 'l1', available)).toEqual({
      gameVersion: '1.20.1',
      loaderVersion: 'l1',
    });
  });

  it('resets a version that is gone, with the loader version', () => {
    expect(reconcileGameVersion('1.8', 'l1', available)).toEqual({
      gameVersion: undefined,
      loaderVersion: undefined,
    });
  });

  it('stays empty when nothing is selected', () => {
    expect(reconcileGameVersion(undefined, undefined, available)).toEqual({
      gameVersion: undefined,
      loaderVersion: undefined,
    });
  });
});
