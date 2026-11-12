import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import { PluginAllowedHostsSection } from '../PluginAllowedHostsSection';
import { PluginHostFunctionsSection } from '../PluginHostFunctionsSection';

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

describe('PluginAllowedHostsSection', () => {
  it('should list the hosts and warn when one admits almost any site', () => {
    const container = mount(() => (
      <PluginAllowedHostsSection allowedHosts={['example.com', '*']} />
    ));

    expect(container.textContent).toContain('example.com');
    expect(container.textContent).toContain('Matches almost any site');
  });

  it('should not warn when every host stays on one domain', () => {
    const container = mount(() => (
      <PluginAllowedHostsSection allowedHosts={['*.example.com']} />
    ));

    expect(container.textContent).not.toContain('Matches almost any site');
  });

  it('should say so when the plugin asks for no hosts', () => {
    const container = mount(() => (
      <PluginAllowedHostsSection allowedHosts={[]} />
    ));

    expect(container.textContent).toContain('makes no network requests');
  });
});

describe('PluginHostFunctionsSection', () => {
  it('should warn about running commands when run_command is declared', () => {
    const container = mount(() => (
      <PluginHostFunctionsSection hostFunctions={['run_command', 'log']} />
    ));

    expect(container.textContent).toContain(
      'can run commands on your computer',
    );
  });

  it('should not warn about running commands when it is not declared', () => {
    const container = mount(() => (
      <PluginHostFunctionsSection hostFunctions={['log']} />
    ));

    expect(container.textContent).not.toContain(
      'can run commands on your computer',
    );
  });
});
