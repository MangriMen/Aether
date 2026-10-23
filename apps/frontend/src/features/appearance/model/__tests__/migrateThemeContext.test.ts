import { describe, expect, it } from 'vitest';

import { migrateThemeContext } from '../migrateThemeContext';
import { DEFAULT_THEME_CONTEXT_VALUE } from '../themeConstants';

describe('migrateThemeContext', () => {
  it('returns the defaults for a missing or non-object value', () => {
    expect(migrateThemeContext(undefined)).toEqual(DEFAULT_THEME_CONTEXT_VALUE);
    expect(migrateThemeContext('dark')).toEqual(DEFAULT_THEME_CONTEXT_VALUE);
  });

  it('keeps stored values of the right type', () => {
    const result = migrateThemeContext({
      transparency: 0.5,
      disableAnimations: true,
    });

    expect(result.transparency).toBe(0.5);
    expect(result.disableAnimations).toBe(true);
  });

  it('replaces a value of the wrong type with the default', () => {
    const result = migrateThemeContext({
      transparency: 'opaque',
      disableAnimations: 1,
      theme: 42,
    });

    expect(result.transparency).toBe(DEFAULT_THEME_CONTEXT_VALUE.transparency);
    expect(result.disableAnimations).toBe(
      DEFAULT_THEME_CONTEXT_VALUE.disableAnimations,
    );
    expect(result.theme).toBe(DEFAULT_THEME_CONTEXT_VALUE.theme);
  });
});
