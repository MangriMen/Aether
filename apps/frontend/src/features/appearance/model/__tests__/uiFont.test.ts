import { describe, expect, it } from 'vitest';

import {
  MAX_CUSTOM_FONT_FAMILY_LENGTH,
  resolveFontSans,
  sanitizeFontFamily,
  UI_FONTS,
} from '../uiFont';

describe('resolveFontSans', () => {
  it('keeps the default stack for the system font', () => {
    expect(resolveFontSans(UI_FONTS.System, 'Fira Sans')).toBeUndefined();
  });

  it('puts the bundled Inter before the default stack', () => {
    expect(resolveFontSans(UI_FONTS.Inter, '')).toMatch(
      /^'Inter Variable', -apple-system/,
    );
  });

  it('puts the named family before the default stack', () => {
    expect(resolveFontSans(UI_FONTS.Custom, ' Fira Sans ')).toMatch(
      /^'Fira Sans', -apple-system/,
    );
  });

  it('keeps the default stack for a custom font without a name', () => {
    expect(resolveFontSans(UI_FONTS.Custom, '  ')).toBeUndefined();
  });
});

describe('sanitizeFontFamily', () => {
  it('drops what could end the quoted family or the declaration', () => {
    expect(sanitizeFontFamily(String.raw`Evil'; color: red; "x\ `)).toBe(
      'Evil color red x',
    );
  });

  it('cuts a name that is too long', () => {
    expect(
      sanitizeFontFamily('a'.repeat(MAX_CUSTOM_FONT_FAMILY_LENGTH + 20)),
    ).toHaveLength(MAX_CUSTOM_FONT_FAMILY_LENGTH);
  });
});
