import { describe, expect, it } from 'vitest';

import { enDictionary, ruDictionary } from '..';

const flatten = (value: unknown, prefix = ''): string[] =>
  value !== null && typeof value === 'object'
    ? Object.entries(value).flatMap(([k, v]) =>
        flatten(v, prefix ? `${prefix}.${k}` : k),
      )
    : [prefix];

describe('dictionary parity', () => {
  it('en and ru have the same keys', () => {
    const en = new Set(flatten(enDictionary));
    const ru = new Set(flatten(ruDictionary));

    expect({
      missingInRu: [...en].filter((k) => !ru.has(k)),
      missingInEn: [...ru].filter((k) => !en.has(k)),
    }).toEqual({ missingInRu: [], missingInEn: [] });
  });
});
