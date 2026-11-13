import { describe, expect, it } from 'vitest';

import { enDictionary, ruDictionary } from '..';

const flatten = (value: unknown, prefix = ''): string[] =>
  value !== null && typeof value === 'object'
    ? Object.entries(value).flatMap(([k, v]) =>
        flatten(v, prefix ? `${prefix}.${k}` : k),
      )
    : [prefix];

const collectStrings = (value: unknown): string[] => {
  if (typeof value === 'string') {
    return [value];
  }

  if (value !== null && typeof value === 'object') {
    return Object.values(value).flatMap(collectStrings);
  }

  return [];
};

describe('dictionary parity', () => {
  it('en and ru have the same keys', () => {
    const en = new Set(flatten(enDictionary));
    const ru = new Set(flatten(ruDictionary));

    expect({
      missingInRu: [...en].filter((k) => !ru.has(k)),
      missingInEn: [...ru].filter((k) => !en.has(k)),
    }).toEqual({ missingInRu: [], missingInEn: [] });
  });

  it('should_not_contain_invisible_characters_when_dictionaries_are_loaded', () => {
    const invisible = /[​-‍﻿]/;
    const offenders = [enDictionary, ruDictionary].flatMap((dictionary) =>
      collectStrings(dictionary).filter((value) => invisible.test(value)),
    );

    expect(offenders).toEqual([]);
  });
});
