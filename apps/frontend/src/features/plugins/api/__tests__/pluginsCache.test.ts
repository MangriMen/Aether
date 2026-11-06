import { QueryClient } from '@tanstack/solid-query';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { Plugin } from '../../model/plugin';

const commands = vi.hoisted(() => ({
  list: vi.fn(),
  get: vi.fn(),
}));

vi.mock('@/platform/ipc', () => ({
  pluginsCommands: commands,
  importersCommands: {},
}));

import { pluginKeys } from '../pluginKeys';
import { pluginsCache, pluginsQueries } from '../pluginsCache';

const plugin = (id: string, restartNeeded = false): Plugin =>
  ({
    manifest: { metadata: { id, name: id } },
    state: { type: 'loaded' },
    restart_needed: restartNeeded,
  }) as unknown as Plugin;

const deferred = <T>() => {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => {
    resolve = r;
  });

  return { promise, resolve };
};

describe('plugins cache', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient();
  });

  it('seeds detail(id) from the answer of the list', async () => {
    commands.list.mockResolvedValue([plugin('a'), plugin('b')]);

    await queryClient.fetchQuery(pluginsQueries.list());

    expect(pluginsCache.getData.detail(queryClient, 'a')).toEqual(plugin('a'));
    expect(pluginsCache.getData.detail(queryClient, 'b')).toEqual(plugin('b'));
    expect(commands.get).not.toHaveBeenCalled();
  });

  it('asks for one plugin only and patches the list, without making it look fresher', async () => {
    commands.list.mockResolvedValue([plugin('a'), plugin('b')]);
    await queryClient.fetchQuery(pluginsQueries.list());
    const listUpdatedAt = queryClient.getQueryState(
      pluginKeys.list(),
    )!.dataUpdatedAt;
    commands.get.mockResolvedValue(plugin('a', true));

    await new Promise((r) => setTimeout(r, 5));
    await pluginsCache.refresh.plugin(queryClient, 'a');

    const list = pluginsCache.getData.list(queryClient)!;
    expect(list.find((p) => p.manifest.metadata.id === 'a')).toEqual(
      plugin('a', true),
    );
    expect(list.find((p) => p.manifest.metadata.id === 'b')).toEqual(
      plugin('b'),
    );
    expect(commands.list).toHaveBeenCalledTimes(1);
    expect(commands.get).toHaveBeenCalledTimes(1);
    expect(queryClient.getQueryState(pluginKeys.list())!.dataUpdatedAt).toBe(
      listUpdatedAt,
    );
  });

  it('keeps a newer list over a get that began before it arrived', async () => {
    commands.list.mockResolvedValueOnce([plugin('a')]);
    await queryClient.fetchQuery(pluginsQueries.list());
    const slowGet = deferred<Plugin>();
    commands.get.mockReturnValue(slowGet.promise);

    const refresh = pluginsCache.refresh.plugin(queryClient, 'a');
    await new Promise((r) => setTimeout(r, 5));
    commands.list.mockResolvedValueOnce([plugin('a', true)]);
    await queryClient.fetchQuery({ ...pluginsQueries.list(), staleTime: 0 });
    slowGet.resolve(plugin('a', false));
    await refresh;

    expect(pluginsCache.getData.list(queryClient)![0]).toEqual(
      plugin('a', true),
    );
  });

  it('does not let a list that began earlier overwrite a newer detail', async () => {
    const slowList = deferred<Plugin[]>();
    commands.list.mockReturnValue(slowList.promise);
    const listing = queryClient.fetchQuery(pluginsQueries.list());
    await new Promise((r) => setTimeout(r, 5));

    commands.get.mockResolvedValue(plugin('a', true));
    await pluginsCache.refresh.plugin(queryClient, 'a');
    slowList.resolve([plugin('a', false)]);
    await listing;

    expect(pluginsCache.getData.detail(queryClient, 'a')).toEqual(
      plugin('a', true),
    );
  });

  it('drops the detail of a removed plugin', async () => {
    commands.list.mockResolvedValue([plugin('a')]);
    await queryClient.fetchQuery(pluginsQueries.list());

    await pluginsCache.remove.plugin(queryClient, 'a');

    expect(pluginsCache.getData.detail(queryClient, 'a')).toBeUndefined();
  });

  it('settles through the list when the plugin cannot be read any more', async () => {
    commands.list.mockResolvedValue([plugin('a')]);
    await queryClient.fetchQuery(pluginsQueries.list());
    commands.get.mockRejectedValue(new Error('gone'));

    await pluginsCache.refresh.plugin(queryClient, 'a');

    expect(pluginsCache.getData.detail(queryClient, 'a')).toBeUndefined();
    // Nothing observes the list here, so it is marked for the next read.
    expect(queryClient.getQueryState(pluginKeys.list())!.isInvalidated).toBe(
      true,
    );
  });
});
