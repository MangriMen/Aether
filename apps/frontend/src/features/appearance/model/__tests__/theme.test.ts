import { describe, expect, it } from 'vitest';

import type { TFunction } from '@/shared/i18n';

import { getThemeLabel } from '../theme';

const t = ((key: string) => `t:${key}`) as unknown as TFunction;

describe('getThemeLabel', () => {
  it('should_translate_the_label_when_the_theme_has_a_label_key', () => {
    expect(
      getThemeLabel(
        { value: 'dark', name: 'Dark', labelKey: 'dark', mode: 'dark' },
        t,
      ),
    ).toBe('t:settings.themes.dark');
  });

  it('should_use_the_theme_name_as_is_when_it_has_no_label_key', () => {
    expect(
      getThemeLabel({ value: 'my-theme', name: 'My Theme', mode: 'dark' }, t),
    ).toBe('My Theme');
  });
});
