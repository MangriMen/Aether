import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import type { AllowedPathValue } from '../../model/pluginSettingsValidation';

import { AllowedPath } from '../AllowedPath';

let dispose: (() => void) | undefined;

const mount = (value: AllowedPathValue) => {
  const container = document.createElement('div');
  document.body.append(container);

  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <AllowedPath value={value} />
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

describe('AllowedPath', () => {
  it('should show a warning with the reason when the folder was ignored', () => {
    const container = mount({
      hostPath: '/a',
      pluginPath: '/cache',
      readOnly: true,
      ignoredReason: 'plugin_reserved',
    });

    const warning = container.querySelector('svg[aria-label]');

    expect(warning?.getAttribute('aria-label')).toContain(
      'taken by a folder the launcher gives every plugin',
    );
  });

  it('should show no warning when the folder is used', () => {
    const container = mount({
      hostPath: '/a',
      pluginPath: '/b',
      readOnly: true,
    });

    expect(container.querySelector('svg[aria-label]')).toBeNull();
  });
});
