import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import { ErrorDetails } from '../ErrorDetails';

let dispose: (() => void) | undefined;

const mount = (defaultOpen?: boolean) => {
  const container = document.createElement('div');
  document.body.append(container);

  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <ErrorDetails details='raw failure text' defaultOpen={defaultOpen} />
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
});

describe('ErrorDetails', () => {
  it('should_hide_the_text_until_it_is_opened', () => {
    const container = mount();

    expect(container.textContent).toContain('Details');
    expect(container.textContent).not.toContain('raw failure text');
  });

  it('should_show_the_text_and_a_copy_button_when_open_by_default', () => {
    const container = mount(true);

    expect(container.textContent).toContain('raw failure text');
    expect(container.textContent).toContain('Copy');
  });
});
