import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import type {
  Plugin,
  PluginUpdateDiff,
  PluginUpdatePreview,
} from '../../model/plugin';

const state = vi.hoisted(() => ({
  preview: {
    data: undefined as unknown,
    isFetching: false,
    error: null as Error | null,
  },
  previewArgs: [] as [string, string][],
  refetch: vi.fn(),
  mutateAsync: vi.fn(),
  pending: false,
}));

vi.mock('../../api/pluginQueries', () => ({
  usePluginUpdatePreview: (id: () => string, tag: () => string) => {
    state.previewArgs.push([id(), tag()]);

    return {
      get data() {
        return state.preview.data;
      },
      get isFetching() {
        return state.preview.isFetching;
      },
      get error() {
        return state.preview.error;
      },
      refetch: state.refetch,
    };
  },
  useUpdatePlugin: () => ({
    mutateAsync: state.mutateAsync,
    get isPending() {
      return state.pending;
    },
  }),
  useIsUpdatingPlugin: () => () => state.pending,
}));

const { PluginUpdateDialog } = await import('../PluginUpdateDialog');

const plugin = {
  manifest: { metadata: { id: 'packwiz', name: 'Packwiz' } },
} as unknown as Plugin;

const noChanges = (): PluginUpdateDiff => ({
  added_host_functions: [],
  removed_host_functions: [],
  added_risks: [],
  added_allowed_hosts: [],
  removed_allowed_hosts: [],
  added_capabilities: [],
  removed_capabilities: [],
  changed_capabilities: [],
  adds_risk: false,
  is_empty: true,
});

const preview = (
  diff: PluginUpdateDiff = noChanges(),
  tag = 'v2',
): PluginUpdatePreview => ({
  release: {
    tag,
    manifest: {
      id: 'packwiz',
      name: 'Packwiz',
      version: '2.0.0',
      description: null,
      authors: [],
      license: null,
      api_version: '^0.5.0',
      allowed_hosts: ['*'],
      wide_allowed_hosts: ['*'],
      host_functions: ['run_command'],
      host_function_risks: ['runs_programs'],
    },
    capabilities: null,
    archive_sha256: 'abc123',
  },
  installed_version: '1.0.0',
  diff,
});

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
};

const confirmButton = () =>
  [...document.querySelectorAll('button')].find(
    (b) => b.textContent?.trim() === 'Update',
  );

describe('PluginUpdateDialog', () => {
  beforeEach(() => {
    state.preview = { data: undefined, isFetching: false, error: null };
    state.previewArgs = [];
    state.refetch.mockReset();
    state.mutateAsync.mockReset();
    state.mutateAsync.mockResolvedValue(null);
    state.pending = false;
  });

  afterEach(() => {
    dispose?.();
    dispose = undefined;
    document.body.innerHTML = '';
  });

  it('should send the tag and hash of the previewed archive when the update is confirmed', async () => {
    state.preview.data = preview();
    const onClose = vi.fn();
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={onClose} />
    ));

    expect(state.previewArgs).toContainEqual(['packwiz', 'v2']);
    confirmButton()?.click();
    await Promise.resolve();
    await Promise.resolve();

    expect(state.mutateAsync).toHaveBeenCalledWith({
      id: 'packwiz',
      tag: 'v2',
      archiveSha256: 'abc123',
    });
    expect(onClose).toHaveBeenCalled();
  });

  it('should keep the update out of reach while the preview is loading', () => {
    state.preview.isFetching = true;
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={() => {}} />
    ));

    expect(document.body.textContent).toContain('Loading');
    expect(confirmButton()?.disabled).toBe(true);
  });

  it('should keep the update out of reach while an older preview is read again', () => {
    state.preview.data = preview();
    state.preview.isFetching = true;
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={() => {}} />
    ));

    confirmButton()?.click();

    expect(confirmButton()?.disabled).toBe(true);
    expect(state.mutateAsync).not.toHaveBeenCalled();
  });

  it('should keep the update out of reach when the preview is of another tag', () => {
    state.preview.data = preview(noChanges(), 'v3');
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={() => {}} />
    ));

    expect(confirmButton()?.disabled).toBe(true);
  });

  it('should warn and list what the new version adds when it adds a risk', () => {
    state.preview.data = preview({
      ...noChanges(),
      added_host_functions: [{ name: 'run_command', risk: 'runs_programs' }],
      added_risks: ['runs_programs'],
      added_allowed_hosts: [{ entry: '*', wide: true }],
      removed_allowed_hosts: ['api.example.com'],
      added_capabilities: [{ kind: 'importer', id: 'mrpack', name: 'Mrpack' }],
      adds_risk: true,
      is_empty: false,
    });
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={() => {}} />
    ));

    const diff = document.querySelector('[data-testid="plugin-update-diff"]');
    const text = diff?.textContent ?? '';

    expect(text).toContain('asks for more than the installed one');
    expect(text).toContain('can ask to run a program');
    expect(text).toContain('Matches almost any site');
    expect(text).toContain('run_command');
    expect(text).toContain('api.example.com');
    expect(text).toContain('Mrpack (importer)');
    expect(document.body.textContent).toContain('1.0.0');
    expect(document.body.textContent).toContain('2.0.0');
    expect(confirmButton()?.disabled).toBe(false);
  });

  it('should say nothing changes when both versions ask for the same', () => {
    state.preview.data = preview();
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={() => {}} />
    ));

    expect(document.body.textContent).toContain(
      'asks for the same as the installed one',
    );
    expect(document.body.textContent).not.toContain(
      'asks for more than the installed one',
    );
  });

  it('should offer no update when the preview fails', () => {
    state.preview.error = new Error('network');
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={() => {}} />
    ));

    expect(document.body.textContent).toContain(
      'Could not read the new version',
    );
    expect(confirmButton()?.disabled).toBe(true);
  });

  it('should read the preview again and stay open when the update fails', async () => {
    state.preview.data = preview();
    state.mutateAsync.mockRejectedValue(new Error('changed'));
    const onClose = vi.fn();
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={onClose} />
    ));

    confirmButton()?.click();
    await vi.waitFor(() => expect(state.refetch).toHaveBeenCalled());

    expect(onClose).not.toHaveBeenCalled();
  });

  it('should offer no update when reading the preview again failed after an earlier read', () => {
    state.preview.data = preview();
    state.preview.error = new Error('network');
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={() => {}} />
    ));

    confirmButton()?.click();

    expect(confirmButton()?.disabled).toBe(true);
    expect(document.body.textContent).toContain(
      'Could not read the new version',
    );
    expect(state.mutateAsync).not.toHaveBeenCalled();
  });

  it('should stay open and keep cancel out of reach while the update runs', () => {
    state.preview.data = preview();
    state.pending = true;
    const onClose = vi.fn();
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={onClose} />
    ));

    const cancel = [...document.querySelectorAll('button')].find(
      (b) => b.textContent?.trim() === 'Cancel',
    );
    expect(cancel?.disabled).toBe(true);

    document.activeElement?.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }),
    );
    document.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }),
    );

    expect(onClose).not.toHaveBeenCalled();
    expect(confirmButton()).toBeDefined();
  });

  it('should close on escape when no update runs', () => {
    state.preview.data = preview();
    const onClose = vi.fn();
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={onClose} />
    ));

    document.activeElement?.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }),
    );

    expect(onClose).toHaveBeenCalled();
  });

  it('should list a changed capability and the capabilities of the new version', () => {
    state.preview.data = {
      ...preview({
        ...noChanges(),
        changed_capabilities: [
          { kind: 'content_provider', id: 'mods', name: 'Mods' },
        ],
        is_empty: false,
      }),
    };
    (state.preview.data as PluginUpdatePreview).release.capabilities = {
      importers: [],
      updaters: [],
      contentProviders: [],
    } as unknown as PluginUpdatePreview['release']['capabilities'];
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag='v2' onClose={() => {}} />
    ));

    expect(document.body.textContent).toContain(
      'Capabilities that work differently now',
    );
    expect(document.body.textContent).toContain('Mods (content provider)');
  });

  it('should show nothing when no release is picked', () => {
    mount(() => (
      <PluginUpdateDialog plugin={plugin} tag={null} onClose={() => {}} />
    ));

    expect(confirmButton()).toBeUndefined();
    expect(state.previewArgs).toHaveLength(0);
  });
});
