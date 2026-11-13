import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import { ProgressMenuButton } from '../ProgressMenuButton';
import { ProgressPopover } from '../ProgressPopover';

let dispose: (() => void) | undefined;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('ProgressMenuButton', () => {
  it('should_be_named_after_the_ongoing_tasks_when_rendered', () => {
    dispose = render(
      () => (
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <ProgressMenuButton popoverComponent={ProgressPopover} />
        </I18nProvider>
      ),
      document.body,
    );

    expect(document.querySelector('button')?.getAttribute('aria-label')).toBe(
      'Ongoing tasks',
    );
  });
});
