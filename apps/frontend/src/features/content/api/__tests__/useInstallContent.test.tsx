import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type * as Settings from '@/features/settings';
import type * as UiKit from '@/shared/ui-kit';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const installContent = vi.hoisted(() => vi.fn());
const showToast = vi.hoisted(() => vi.fn());

vi.mock('@/platform/ipc', () => ({
  instanceCommands: { installContent },
}));

vi.mock('@/features/settings', async (importOriginal) => ({
  ...(await importOriginal<typeof Settings>()),
  useSettingsSearchParams: () => ({ open: vi.fn() }),
}));

vi.mock('@/shared/ui-kit', async (importOriginal) => ({
  ...(await importOriginal<typeof UiKit>()),
  showToast,
}));

import { useInstallContent } from '../contentQueries';

let dispose: (() => void) | undefined;

const mountInstall = () => {
  let install!: ReturnType<typeof useInstallContent>;
  const container = document.createElement('div');
  document.body.append(container);
  dispose = render(
    () => (
      <QueryClientProvider client={new QueryClient()}>
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          {(() => {
            install = useInstallContent();
            return null;
          })()}
        </I18nProvider>
      </QueryClientProvider>
    ),
    container,
  );

  return install;
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  vi.clearAllMocks();
  document.body.replaceChildren();
});

describe('useInstallContent', () => {
  it('installs a modpack and reports success', async () => {
    installContent.mockResolvedValue(undefined);
    const payload = {
      type: 'modpack',
      data: {
        contentId: 'pack',
        contentVersion: 'v1',
        providerId: { pluginId: 'aether', capabilityId: 'modrinth' },
      },
    } as const;

    await mountInstall().mutateAsync(payload);

    expect(installContent).toHaveBeenCalledWith(payload);
    expect(showToast).toHaveBeenCalledWith(
      expect.objectContaining({ variant: 'success' }),
    );
  });
});
