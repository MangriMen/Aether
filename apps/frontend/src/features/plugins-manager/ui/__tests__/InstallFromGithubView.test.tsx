import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const commands = vi.hoisted(() => ({
  listPluginReleases: vi.fn(),
  previewPluginRelease: vi.fn(),
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

const listing = { owner: 'o', repo: 'r', releases, default_tag: 'v2' };

const previewOf = (tag: string) => ({
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
    wide_allowed_hosts: [],
    host_functions: null,
    host_function_risks: [],
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
  commands.listPluginReleases.mockResolvedValue(listing);
  commands.previewPluginRelease.mockImplementation(
    (_provider: string, _url: string, tag: string) =>
      Promise.resolve(previewOf(tag)),
  );
  commands.installPluginFromProvider.mockResolvedValue('p');
});

afterEach(() => {
  vi.unstubAllGlobals();
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

const mountView = () => {
  const container = document.createElement('div');
  document.body.append(container);
  dispose = render(
    () => (
      <QueryClientProvider client={new QueryClient()}>
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <InstallFromGithubView />
        </I18nProvider>
      </QueryClientProvider>
    ),
    container,
  );

  return container;
};

const typeUrl = (container: HTMLElement, url: string) => {
  const input = container.querySelector('input')!;
  input.value = url;
  input.dispatchEvent(new InputEvent('input', { bubbles: true }));
};

const pickTag = async (tag: string) => {
  document
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
  ].find((o) => o.textContent?.includes(tag))!;

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
};

describe('InstallFromGithubView', () => {
  it('should_not_preview_a_new_url_with_the_tag_of_the_previous_one', async () => {
    commands.listPluginReleases.mockImplementation(
      (_provider: string, url: string) =>
        Promise.resolve(
          url.endsWith('/o/r')
            ? listing
            : {
                ...listing,
                releases: [release('b2'), release('b1')],
                default_tag: 'b2',
              },
        ),
    );
    const container = mountView();

    typeUrl(container, 'https://github.com/o/r');
    await vi.waitFor(() => {
      expect(findButton(container, 'Install')?.disabled).toBe(false);
    });
    await pickTag('v1');
    await vi.waitFor(() => {
      expect(commands.previewPluginRelease).toHaveBeenLastCalledWith(
        'git_hub',
        'https://github.com/o/r',
        'v1',
      );
    });

    typeUrl(container, 'https://github.com/o/b');
    await vi.waitFor(() => {
      expect(commands.previewPluginRelease).toHaveBeenLastCalledWith(
        'git_hub',
        'https://github.com/o/b',
        'b2',
      );
    });

    const previewedNew = commands.previewPluginRelease.mock.calls.filter(
      ([, url]: string[]) => url === 'https://github.com/o/b',
    );
    expect(previewedNew.map(([, , tag]: string[]) => tag)).toEqual(['b2']);
  });

  it('should_preview_the_picked_tag_and_install_with_the_hash_of_that_preview', async () => {
    const container = mountView();

    typeUrl(container, 'https://github.com/o/r');

    await vi.waitFor(() => {
      expect(commands.listPluginReleases).toHaveBeenCalledWith(
        'git_hub',
        'https://github.com/o/r',
      );
      expect(commands.previewPluginRelease).toHaveBeenCalledWith(
        'git_hub',
        'https://github.com/o/r',
        'v2',
      );
      expect(findButton(container, 'Install')?.disabled).toBe(false);
    });

    await pickTag('v1');

    await vi.waitFor(() => {
      expect(commands.previewPluginRelease).toHaveBeenLastCalledWith(
        'git_hub',
        'https://github.com/o/r',
        'v1',
      );
      expect(findButton(container, 'Install')?.disabled).toBe(false);
    });

    findButton(container, 'Install')!.click();

    expect(commands.listPluginReleases).toHaveBeenCalledTimes(1);

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
