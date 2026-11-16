import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import type { ModdedLoaderVersion } from '@/features/minecraft';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import { SelectSpecificLoaderVersion } from '../SelectSpecificLoaderVersion';

let dispose: (() => void) | undefined;

const options = [
  { id: '0.15.0', stable: true },
  { id: '0.16.0', stable: true },
] as unknown as ModdedLoaderVersion[];

const mount = (value: ModdedLoaderVersion | null) => {
  const container = document.createElement('div');
  document.body.append(container);
  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <SelectSpecificLoaderVersion
          options={options}
          value={value}
          placeholder='Pick a version'
        />
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

describe('SelectSpecificLoaderVersion', () => {
  it('should show the controlled value', () => {
    expect(mount(options[1] ?? null).textContent).toContain('0.16.0');
  });

  it('should show the placeholder when the form value is empty', () => {
    const container = mount(null);

    expect(container.textContent).toContain('Pick a version');
    expect(container.textContent).not.toContain('0.15.0');
  });
});
