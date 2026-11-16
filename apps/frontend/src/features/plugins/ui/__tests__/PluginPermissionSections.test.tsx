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
  it('should list the hosts and warn when the core reports a wide one', () => {
    const container = mount(() => (
      <PluginAllowedHostsSection
        allowedHosts={['example.com', '*']}
        wideHosts={['*']}
      />
    ));

    expect(container.textContent).toContain('example.com');
    expect(container.textContent).toContain('Matches almost any site');
  });

  it('should not warn when the core reports no wide host', () => {
    const container = mount(() => (
      <PluginAllowedHostsSection
        allowedHosts={['*.example.com']}
        wideHosts={[]}
      />
    ));

    expect(container.textContent).not.toContain('Matches almost any site');
  });

  it('should say so when the plugin asks for no hosts', () => {
    const container = mount(() => (
      <PluginAllowedHostsSection allowedHosts={[]} wideHosts={[]} />
    ));

    expect(container.textContent).toContain('makes no network requests');
  });
});

describe('PluginHostFunctionsSection', () => {
  it('should warn about each risk the core reports', () => {
    const container = mount(() => (
      <PluginHostFunctionsSection
        hostFunctions={['run_command', 'write_instance_file', 'install_java']}
        risks={['runs_programs', 'writes_instance_files', 'installs_java']}
      />
    ));

    expect(container.textContent).toContain(
      'can ask to run a program on your computer',
    );
    expect(container.textContent).toContain(
      'put files into your game instances',
    );
    expect(container.textContent).toContain('install a Java runtime');
  });

  it('should not warn when the core reports no risk', () => {
    const container = mount(() => (
      <PluginHostFunctionsSection hostFunctions={['log']} risks={[]} />
    ));

    expect(container.textContent).not.toContain('can ask to run a program');
    expect(container.textContent).not.toContain('put files into your game');
  });
});
