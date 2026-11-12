import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const commands = vi.hoisted(() => ({
  previewPluginFromProvider: vi.fn(),
  installPluginFromProvider: vi.fn(),
}));

vi.mock('@/platform/ipc', async (importOriginal) => ({
  ...(await importOriginal<object>()),
  pluginsCommands: commands,
}));

import { InstallFromGithubView } from '../InstallFromGithubView';

const release = (tag: string) => ({
  tag_name: tag,
  version: tag,
  is_prerelease: false,
  published_at: '',
  html_url: '',
  download_url: '',
});

const releases = [release('v2'), release('v1')];

const previewOf = (tag: string) => ({
  owner: 'o',
  repo: 'r',
  releases,
  tag,
  manifest: {
    id: 'p',
    name: 'Plugin',
    version: tag,
    description: null,
    authors: [],
    license: null,
    api_version: null,
    allowed_hosts: [],
    host_functions: null,
  },
  capabilities: null,
  archive_sha256: `sha-of-${tag}`,
  size: 1,
});

let dispose: (() => void) | undefined;

const findButton = (container: HTMLElement, text: string) =>
  [...container.querySelectorAll('button')].find(
    (b) => b.textContent?.trim() === text,
  );

beforeEach(() => {
  vi.resetAllMocks();
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
  commands.previewPluginFromProvider.mockImplementation(
    (_provider: string, _url: string, tag: string | null) =>
      Promise.resolve(previewOf(tag ?? 'v2')),
  );
  commands.installPluginFromProvider.mockResolvedValue('p');
});

afterEach(() => {
  vi.unstubAllGlobals();
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('InstallFromGithubView', () => {
  it('should_preview_the_picked_tag_and_install_with_the_hash_of_that_preview', async () => {
    const container = document.createElement('div');
    document.body.append(container);
    dispose = render(
      () => (
        <QueryClientProvider client={new QueryClient()}>
          <I18nProvider
            resources={LOCALE_RESOURCES}
            fallbackLocale={LOCALES.En}
          >
            <InstallFromGithubView />
          </I18nProvider>
        </QueryClientProvider>
      ),
      container,
    );

    const input = container.querySelector('input')!;
    input.value = 'https://github.com/o/r';
    input.dispatchEvent(new InputEvent('input', { bubbles: true }));

    await vi.waitFor(() => {
      expect(commands.previewPluginFromProvider).toHaveBeenCalledWith(
        'git_hub',
        'https://github.com/o/r',
        null,
      );
      expect(findButton(container, 'Install')?.disabled).toBe(false);
    });

    container
      .querySelector<HTMLElement>('[role="combobox"], button[aria-haspopup]')!
      .dispatchEvent(
        new PointerEvent('pointerdown', {
          bubbles: true,
          button: 0,
          pointerType: 'mouse',
        }),
      );
    await vi.waitFor(() => {
      expect(document.querySelector('[role="option"]')).not.toBeNull();
    });
    const option = [
      ...document.querySelectorAll<HTMLElement>('[role="option"]'),
    ].find((o) => o.textContent?.includes('v1'))!;

    for (const type of ['pointerdown', 'pointerup']) {
      option.dispatchEvent(
        new PointerEvent(type, {
          bubbles: true,
          button: 0,
          pointerType: 'mouse',
        }),
      );
    }

    option.click();

    await vi.waitFor(() => {
      expect(commands.previewPluginFromProvider).toHaveBeenLastCalledWith(
        'git_hub',
        'https://github.com/o/r',
        'v1',
      );
      expect(findButton(container, 'Install')?.disabled).toBe(false);
    });

    findButton(container, 'Install')!.click();

    await vi.waitFor(() => {
      expect(commands.installPluginFromProvider).toHaveBeenCalledWith(
        'git_hub',
        'https://github.com/o/r',
        'v1',
        'sha-of-v1',
      );
    });
  });
});
