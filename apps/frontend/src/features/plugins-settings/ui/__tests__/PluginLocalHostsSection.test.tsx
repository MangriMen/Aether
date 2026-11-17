import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import { PluginLocalHostsSection } from '../PluginLocalHostsSection';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  const container = document.createElement('div');
  document.body.append(container);
  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        {ui()}
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

describe('PluginLocalHostsSection', () => {
  it('should_list_each_local_address_and_take_back_the_one_pressed', () => {
    const onRevoke = vi.fn();
    const container = mount(() => (
      <PluginLocalHostsSection
        localHosts={['192.168.1.10:8080', '[::1]:25565']}
        onRevoke={onRevoke}
      />
    ));

    const items = container.querySelectorAll('li');
    expect(items).toHaveLength(2);
    expect(items[0]!.textContent).toContain('192.168.1.10:8080');
    expect(items[1]!.textContent).toContain('[::1]:25565');

    items[1]!.querySelector('button')!.click();

    expect(onRevoke).toHaveBeenCalledExactlyOnceWith('[::1]:25565');
  });

  it('should_not_take_anything_back_while_disabled', () => {
    const onRevoke = vi.fn();
    const container = mount(() => (
      <PluginLocalHostsSection
        localHosts={['localhost:443']}
        onRevoke={onRevoke}
        disabled
      />
    ));

    const button = container.querySelector('button')!;
    expect(button.disabled).toBe(true);
    button.click();
    expect(onRevoke).not.toHaveBeenCalled();
  });
});
