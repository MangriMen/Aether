import { describe, expect, it } from 'vitest';

import { createI18nError, parseI18nError } from '../i18nError';

describe('parseI18nError', () => {
  it('should return the key and values when the error was created by createI18nError', () => {
    const raw = createI18nError('errors.some', { count: 2 });

    expect(parseI18nError(raw)).toEqual({
      key: 'errors.some',
      values: { count: 2 },
    });
  });

  it('should return undefined when the text is not JSON', () => {
    expect(parseI18nError('plain message')).toBeUndefined();
  });

  it('should return undefined when the JSON has no string key', () => {
    expect(parseI18nError('{"key": 5}')).toBeUndefined();
    expect(parseI18nError('null')).toBeUndefined();
  });
});
