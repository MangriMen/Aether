import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import type { ContentListProps } from '../ContentList';

import { ContentList } from '../ContentList';

const open = vi.hoisted(() => vi.fn());

vi.mock('@/features/settings', () => ({
  SettingsTab: { Plugins: 'plugins' },
  useSettingsSearchParams: () => ({ open }),
}));

let dispose: (() => void) | undefined;

const mount = (props: ContentListProps) => {
  const container = document.createElement('div');
  document.body.append(container);

  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <ContentList {...props} />
      </I18nProvider>
    ),
    container,
  );

  return container;
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
  open.mockClear();
});

describe('ContentList', () => {
  it('should_explain_that_providers_failed_when_the_providers_query_errored', () => {
    const container = mount({ isProvidersError: true });

    expect(container.textContent).toContain('Failed to load content providers');
  });

  it('should_offer_plugin_settings_when_there_are_no_providers', () => {
    const container = mount({ hasNoProviders: true });

    container.querySelector('button')?.click();

    expect(container.textContent).toContain('No content providers');
    expect(open).toHaveBeenCalledWith('plugins');
  });

  it('should_say_nothing_was_found_when_items_are_not_loaded_yet', () => {
    const container = mount({});

    expect(container.textContent).toContain('No results found');
  });
});
