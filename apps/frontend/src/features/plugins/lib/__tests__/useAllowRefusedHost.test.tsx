import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { LauncherError } from '@/features/errors';
import type * as UiKit from '@/shared/ui-kit';

import { findErrorAction } from '@/features/errors';
import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const commands = vi.hoisted(() => ({
  grantHostAccess: vi.fn(),
}));

vi.mock('@/platform/ipc', () => ({
  pluginsCommands: commands,
  importersCommands: {},
}));

vi.mock('@/shared/ui-kit', async (importOriginal) => ({
  ...(await importOriginal<typeof UiKit>()),
  showToast: vi.fn(),
}));

import { useAllowRefusedHost } from '../useAllowRefusedHost';

const refused = (host: string) =>
  ({
    type: 'plugin',
    payload: {
      code: 'HOST_NOT_ALLOWED',
      payload: {
        pluginId: 'packwiz',
        host,
        cause: { type: 'unknown', details: 'cdn refused' },
      },
    },
  }) as LauncherError;

const deferred = () => {
  let resolve!: (value: unknown) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });

  return { promise, resolve, reject };
};

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
          useAllowRefusedHost();
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
  commands.grantHostAccess.mockReset();
});

describe('useAllowRefusedHost', () => {
  it('should_allow_the_refused_host_and_stay_off_once_it_is_allowed', async () => {
    const grant = deferred();
    commands.grantHostAccess.mockReturnValue(grant.promise);
    mountHook();
    const button = buttonOf(refused('cdn.example'));

    expect(button.textContent).toBe('Allow cdn.example');

    button.click();

    await vi.waitFor(() => expect(isBusy(button)).toBe(true));
    expect(commands.grantHostAccess).toHaveBeenCalledWith(
      'packwiz',
      'https://cdn.example/',
      false,
    );

    grant.resolve({ type: 'granted', host: 'cdn.example' });

    await vi.waitFor(() => expect(isBusy(button)).toBe(false));
    expect(button.disabled).toBe(true);
  });

  it('should_offer_the_button_again_when_the_grant_failed', async () => {
    const grant = deferred();
    commands.grantHostAccess.mockReturnValue(grant.promise);
    mountHook();
    const button = buttonOf(refused('cdn.example'));

    button.click();
    await vi.waitFor(() => expect(isBusy(button)).toBe(true));
    grant.reject(new Error('refused'));

    await vi.waitFor(() => expect(isBusy(button)).toBe(false));
    expect(button.disabled).toBe(false);
  });

  it('should_keep_the_brackets_of_an_ipv6_host', () => {
    mountHook();

    expect(findErrorAction(refused('[2001:db8::1]'))).toBeDefined();
  });

  it.each([
    ['a path', 'cdn.example/x'],
    ['a query', 'cdn.example?x'],
    ['a fragment', 'cdn.example#x'],
    ['a backslash', 'cdn.example\\x'],
    ['a port', 'cdn.example:8080'],
    ['unicode', 'пример.рф'],
    ['upper case', 'CDN.example'],
    ['nothing', ''],
  ])(
    'should_offer_nothing_when_the_host_has_%s_and_the_grant_would_name_another',
    (_case, host) => {
      mountHook();

      expect(findErrorAction(refused(host))).toBeUndefined();
    },
  );

  it('should_offer_nothing_for_another_error_or_once_the_app_is_gone', () => {
    mountHook();

    expect(
      findErrorAction({
        type: 'plugin',
        payload: { code: 'NOT_FOUND', payload: { pluginId: 'packwiz' } },
      }),
    ).toBeUndefined();

    disposers.splice(0).forEach((dispose) => dispose());

    expect(findErrorAction(refused('cdn.example'))).toBeUndefined();
  });
});
