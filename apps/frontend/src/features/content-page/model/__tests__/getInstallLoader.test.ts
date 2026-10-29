import { describe, expect, it } from 'vitest';

import { getInstallLoader } from '../getInstallLoader';

describe('getInstallLoader', () => {
  it('takes the instance loader when the content is a mod', () => {
    expect(getInstallLoader('mod', 'fabric', 'forge')).toBe('fabric');
  });

  it('takes the filter loader when the instance has none', () => {
    expect(getInstallLoader('mod', undefined, 'forge')).toBe('forge');
  });

  it('ignores the filter loader when it is not a loader name', () => {
    expect(getInstallLoader('mod', undefined, 'neo')).toBeUndefined();
  });

  it('has no loader when neither the instance nor the filter gives one', () => {
    expect(getInstallLoader('mod', undefined, undefined)).toBeUndefined();
  });

  it.each(['resourcepack', 'shaderpack', 'datapack'] as const)(
    'has no loader when the content is a %s',
    (contentType) => {
      expect(getInstallLoader(contentType, 'fabric', 'forge')).toBeUndefined();
    },
  );
});
