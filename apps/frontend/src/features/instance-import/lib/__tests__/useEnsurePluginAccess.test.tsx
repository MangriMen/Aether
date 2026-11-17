/* eslint-disable sonarjs/no-hardcoded-ip, sonarjs/no-clear-text-protocols -- packs on a local
   address, over http, are what is under test */
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type * as Dialogs from '@/shared/dialogs';
import type * as UiKit from '@/shared/ui-kit';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const commands = vi.hoisted(() => ({
  grantHostAccess: vi.fn(),
  grantLocalHostAccess: vi.fn(),
  grantFolderAccess: vi.fn(),
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

import { useEnsurePluginAccess } from '../useEnsurePluginAccess';

let dispose: (() => void) | undefined;

/** The check, made inside the providers as the import form makes it. */
const mountEnsure = () => {
  let ensure!: ReturnType<typeof useEnsurePluginAccess>;
  const container = document.createElement('div');
  document.body.append(container);
  dispose = render(
    () => (
      <QueryClientProvider client={new QueryClient()}>
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          {(() => {
            ensure = useEnsurePluginAccess();
            return null;
          })()}
        </I18nProvider>
      </QueryClientProvider>
    ),
    container,
  );

  return ensure;
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';

  for (const command of Object.values(commands)) {
    command.mockReset();
  }

  askConfirmation.mockReset();
});

describe('useEnsurePluginAccess', () => {
  it('should_ask_for_a_local_address_with_its_warning_when_the_pack_is_on_one', async () => {
    commands.grantLocalHostAccess.mockImplementation(
      (_id: string, _url: string, dryRun: boolean) =>
        Promise.resolve({
          type: dryRun ? 'needed' : 'granted',
          host: '192.168.1.10:8080',
        }),
    );
    askConfirmation.mockResolvedValue(true);
    const ensure = mountEnsure();

    await expect(
      ensure('packwiz', 'http://192.168.1.10:8080/pack.toml'),
    ).resolves.toBe(true);

    const [texts] = askConfirmation.mock.calls[0] as [{ description: string }];
    expect(texts.description).toContain(
      '192.168.1.10:8080 is an address in your own network or on this computer',
    );
    expect(commands.grantLocalHostAccess).toHaveBeenLastCalledWith(
      'packwiz',
      'http://192.168.1.10:8080/pack.toml',
      false,
    );
    expect(commands.grantHostAccess).not.toHaveBeenCalled();
  });

  it('should_give_nothing_when_the_user_does_not_allow_the_local_address', async () => {
    commands.grantLocalHostAccess.mockResolvedValue({
      type: 'needed',
      host: 'localhost:25565',
    });
    askConfirmation.mockResolvedValue(false);
    const ensure = mountEnsure();

    await expect(
      ensure('packwiz', 'http://localhost:25565/pack.toml'),
    ).resolves.toBe(false);

    expect(commands.grantLocalHostAccess).toHaveBeenCalledOnce();
    expect(commands.grantHostAccess).not.toHaveBeenCalled();
  });

  it('should_ask_for_an_ordinary_host_when_the_literal_is_no_local_address', async () => {
    commands.grantLocalHostAccess.mockRejectedValue(new Error('not local'));
    commands.grantHostAccess.mockResolvedValue({
      type: 'alreadyAllowed',
      host: '8.8.8.8',
    });
    const ensure = mountEnsure();

    await expect(ensure('packwiz', 'https://8.8.8.8/pack.toml')).resolves.toBe(
      true,
    );

    expect(commands.grantHostAccess).toHaveBeenCalledWith(
      'packwiz',
      'https://8.8.8.8/pack.toml',
      true,
    );
    expect(askConfirmation).not.toHaveBeenCalled();
  });

  it.each(['nas.lan', 'app.localhost'])(
    'should_not_ask_about_a_local_address_when_the_host_is_the_name_%s',
    async (host) => {
      commands.grantHostAccess.mockResolvedValue({
        type: 'alreadyAllowed',
        host,
      });
      const ensure = mountEnsure();

      await expect(ensure('packwiz', `http://${host}/pack.toml`)).resolves.toBe(
        true,
      );

      expect(commands.grantLocalHostAccess).not.toHaveBeenCalled();
    },
  );
});
