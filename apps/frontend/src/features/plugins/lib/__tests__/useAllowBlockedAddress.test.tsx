/* eslint-disable sonarjs/no-hardcoded-ip, sonarjs/no-clear-text-protocols -- a local address,
   and the URL the grant names it by, are what is under test */
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { LauncherError } from '@/features/errors';
import type * as Dialogs from '@/shared/dialogs';
import type * as UiKit from '@/shared/ui-kit';

import { findErrorAction } from '@/features/errors';
import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const commands = vi.hoisted(() => ({
  grantLocalHostAccess: vi.fn(),
}));

const askConfirmation = vi.hoisted(() => vi.fn());

vi.mock('@/platform/ipc', () => ({
  pluginsCommands: commands,
  importersCommands: {},
}));

vi.mock('@/shared/dialogs', async (importOriginal) => ({
  ...(await importOriginal<typeof Dialogs>()),
  askConfirmation,
}));

vi.mock('@/shared/ui-kit', async (importOriginal) => ({
  ...(await importOriginal<typeof UiKit>()),
  showToast: vi.fn(),
}));

import { useAllowBlockedAddress } from '../useAllowBlockedAddress';

const blocked = (host: string, port: number) =>
  ({
    type: 'plugin',
    payload: {
      code: 'ADDRESS_BLOCKED',
      payload: {
        plugin_id: 'packwiz',
        host,
        port,
        cause: { type: 'unknown', details: 'nas refused' },
      },
    },
  }) as LauncherError;

const disposers: (() => void)[] = [];

const mountInto = (code: () => unknown) => {
  const container = document.createElement('div');
  document.body.append(container);
  disposers.push(render(code as never, container));

  return container;
};

/** The hook, called inside the providers as the app calls it. */
const mountHook = () =>
  mountInto(() => (
    <QueryClientProvider client={new QueryClient()}>
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        {(() => {
          useAllowBlockedAddress();
          return null;
        })()}
      </I18nProvider>
    </QueryClientProvider>
  ));

/** The button of the error, rendered outside the providers, as a toast renders it. */
const buttonOf = (error: LauncherError) => {
  const action = findErrorAction(error);
  expect(action).toBeDefined();

  return mountInto(action!).querySelector('button')!;
};

const isBusy = (button: HTMLButtonElement) =>
  button.getAttribute('aria-busy') === 'true';

afterEach(() => {
  for (const dispose of disposers.splice(0)) {
    dispose();
  }

  document.body.innerHTML = '';
  commands.grantLocalHostAccess.mockReset();
  askConfirmation.mockReset();
});

describe('useAllowBlockedAddress', () => {
  it('should_allow_the_address_only_after_the_user_confirms_the_warning', async () => {
    askConfirmation.mockResolvedValue(true);
    commands.grantLocalHostAccess.mockResolvedValue({
      type: 'granted',
      host: '192.168.1.10:8080',
    });
    mountHook();
    const button = buttonOf(blocked('192.168.1.10', 8080));

    expect(button.textContent).toBe('Allow this local address');

    button.click();

    await vi.waitFor(() =>
      expect(commands.grantLocalHostAccess).toHaveBeenCalledWith(
        'packwiz',
        'http://192.168.1.10:8080/',
        false,
      ),
    );
    expect(askConfirmation).toHaveBeenCalledOnce();
    const [texts] = askConfirmation.mock.calls[0] as [
      { header: string; description: string },
    ];
    expect(texts.header).toBe('Let packwiz reach 192.168.1.10:8080?');
    expect(texts.description).toContain(
      '192.168.1.10:8080 is an address in your own network or on this computer',
    );
    expect(texts.description).toContain(
      'packwiz will be able to send requests to it',
    );
    expect(texts.description).not.toContain('is a name');

    await vi.waitFor(() => expect(isBusy(button)).toBe(false));
    expect(button.disabled).toBe(true);
  });

  it.each(['nas.lan', 'app.localhost'])(
    'should_warn_that_the_name_%s_can_later_point_elsewhere',
    async (host) => {
      askConfirmation.mockResolvedValue(false);
      mountHook();

      buttonOf(blocked(host, 8080)).click();

      await vi.waitFor(() => expect(askConfirmation).toHaveBeenCalledOnce());
      const [texts] = askConfirmation.mock.calls[0] as [
        { description: string },
      ];
      expect(texts.description).toContain(
        `${host}:8080 is a name, not a fixed address`,
      );
    },
  );

  it('should_give_nothing_and_offer_the_button_again_when_the_user_does_not_confirm', async () => {
    askConfirmation.mockResolvedValue(false);
    mountHook();
    const button = buttonOf(blocked('localhost', 25565));

    button.click();

    await vi.waitFor(() => expect(askConfirmation).toHaveBeenCalledOnce());
    await vi.waitFor(() => expect(isBusy(button)).toBe(false));
    expect(button.disabled).toBe(false);
    expect(commands.grantLocalHostAccess).not.toHaveBeenCalled();
  });

  it.each([
    ['an IPv6 literal', '[fd12::1]', 443, 'http://[fd12::1]:443/'],
    ['the port of http', 'nas.lan', 80, 'http://nas.lan:80/'],
  ])(
    'should_grant_exactly_the_host_and_port_of_%s',
    async (_case, host, port, url) => {
      askConfirmation.mockResolvedValue(true);
      commands.grantLocalHostAccess.mockResolvedValue({
        type: 'granted',
        host,
      });
      mountHook();

      buttonOf(blocked(host, port)).click();

      await vi.waitFor(() =>
        expect(commands.grantLocalHostAccess).toHaveBeenCalledWith(
          'packwiz',
          url,
          false,
        ),
      );
    },
  );

  it.each([
    ['a path', 'nas.lan/x', 80],
    ['credentials', 'user@nas.lan', 80],
    ['a port in the host', 'nas.lan:8080', 80],
    ['unicode', 'пример.рф', 80],
    ['upper case', 'NAS.lan', 80],
    ['nothing', '', 80],
    ['port zero', 'nas.lan', 0],
    ['a port out of range', 'nas.lan', 65_536],
  ])(
    'should_offer_nothing_when_the_address_has_%s_and_the_grant_would_name_another',
    (_case, host, port) => {
      mountHook();

      expect(findErrorAction(blocked(host, port))).toBeUndefined();
    },
  );

  it('should_offer_nothing_for_a_refused_host_or_once_the_app_is_gone', () => {
    mountHook();

    expect(
      findErrorAction({
        type: 'plugin',
        payload: {
          code: 'HOST_NOT_ALLOWED',
          payload: {
            plugin_id: 'packwiz',
            host: 'cdn.example',
            cause: { type: 'unknown', details: 'x' },
          },
        },
      }),
    ).toBeUndefined();

    disposers.splice(0).forEach((dispose) => dispose());

    expect(findErrorAction(blocked('192.168.1.10', 8080))).toBeUndefined();
  });
});
