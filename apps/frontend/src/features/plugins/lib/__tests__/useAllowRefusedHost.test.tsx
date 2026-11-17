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

const refused = {
  type: 'plugin',
  payload: {
    code: 'HOST_NOT_ALLOWED',
    payload: {
      plugin_id: 'packwiz',
      host: 'cdn.example',
      cause: { type: 'unknown', details: 'cdn refused' },
    },
  },
} as LauncherError;

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

afterEach(() => {
  for (const dispose of disposers.splice(0)) {
    dispose();
  }

  document.body.innerHTML = '';
  commands.grantHostAccess.mockReset();
});

describe('useAllowRefusedHost', () => {
  it('should_allow_the_refused_host_when_the_button_of_the_error_is_pressed', async () => {
    commands.grantHostAccess.mockResolvedValue({
      type: 'granted',
      host: 'cdn.example',
    });
    mountHook();

    const action = findErrorAction(refused);
    expect(action).toBeDefined();

    // Outside the providers, as a toast renders it.
    const button = mountInto(action!).querySelector('button')!;
    expect(button.textContent).toBe('Allow cdn.example');

    button.click();

    await vi.waitFor(() => expect(button.disabled).toBe(true));
    expect(commands.grantHostAccess).toHaveBeenCalledWith(
      'packwiz',
      'https://cdn.example/',
      false,
    );
  });

  it('should_offer_nothing_for_another_error_or_once_the_app_is_gone', () => {
    mountHook();

    expect(
      findErrorAction({
        type: 'plugin',
        payload: { code: 'NOT_FOUND', payload: { plugin_id: 'packwiz' } },
      }),
    ).toBeUndefined();

    disposers.splice(0).forEach((dispose) => dispose());

    expect(findErrorAction(refused)).toBeUndefined();
  });
});
