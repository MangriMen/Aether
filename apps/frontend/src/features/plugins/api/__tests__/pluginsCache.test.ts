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
    vi.resetAllMocks();
    queryClient = new QueryClient();
  });

  it('should_seed_detail_from_the_answer_of_the_list', async () => {
    commands.list.mockResolvedValue([plugin('a'), plugin('b')]);

    await queryClient.fetchQuery(pluginsQueries.list());

    expect(pluginsCache.getData.detail(queryClient, 'a')).toEqual(plugin('a'));
    expect(pluginsCache.getData.detail(queryClient, 'b')).toEqual(plugin('b'));
    expect(commands.get).not.toHaveBeenCalled();
  });

  it('should_ask_for_one_plugin_and_patch_the_list_without_making_it_look_fresher', async () => {
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

  it('should_ask_for_the_list_again_when_it_arrived_after_the_get_began', async () => {
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

    // The list that arrived is not overwritten by the older answer, and is asked for again.
    expect(pluginsCache.getData.list(queryClient)![0]).toEqual(
      plugin('a', true),
    );
    expect(queryClient.getQueryState(pluginKeys.list())!.isInvalidated).toBe(
      true,
    );
  });

  it('should_keep_the_list_marked_for_a_new_read_when_another_plugin_is_refreshed', async () => {
    commands.list.mockResolvedValue([plugin('a'), plugin('b')]);
    await queryClient.fetchQuery(pluginsQueries.list());
    await pluginsCache.invalidate.list(queryClient);
    commands.get.mockResolvedValue(plugin('b', true));

    await pluginsCache.refresh.plugin(queryClient, 'b');

    expect(queryClient.getQueryState(pluginKeys.list())!.isInvalidated).toBe(
      true,
    );
  });

  it('should_wait_out_a_get_already_on_its_way_and_ask_again', async () => {
    commands.list.mockResolvedValue([plugin('a')]);
    await queryClient.fetchQuery(pluginsQueries.list());
    const first = deferred<Plugin>();
    commands.get.mockReturnValueOnce(first.promise);
    commands.get.mockResolvedValueOnce(plugin('a', true));

    const loading = pluginsCache.refresh.plugin(queryClient, 'a');
    await new Promise((r) => setTimeout(r, 5));
    const failed = pluginsCache.refresh.plugin(queryClient, 'a');
    first.resolve(plugin('a', false));
    await Promise.all([loading, failed]);

    expect(commands.get).toHaveBeenCalledTimes(2);
    expect(pluginsCache.getData.detail(queryClient, 'a')).toEqual(
      plugin('a', true),
    );
  });

  it('should_still_know_the_plugin_from_the_list_when_its_detail_is_gone', async () => {
    commands.list.mockResolvedValue([plugin('a')]);
    await queryClient.fetchQuery(pluginsQueries.list());

    queryClient.removeQueries({ queryKey: pluginKeys.detail('a') });

    expect(pluginsCache.getData.detail(queryClient, 'a')).toEqual(plugin('a'));
  });

  it('should_keep_a_newer_detail_in_both_when_a_list_that_began_earlier_arrives', async () => {
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
    expect(pluginsCache.getData.list(queryClient)![0]).toEqual(
      plugin('a', true),
    );
  });

  it('should_drop_the_detail_of_a_removed_plugin', async () => {
    commands.list.mockResolvedValue([plugin('a')]);
    await queryClient.fetchQuery(pluginsQueries.list());

    await pluginsCache.remove.plugin(queryClient, 'a');

    expect(queryClient.getQueryData(pluginKeys.detail('a'))).toBeUndefined();
  });

  it('should_settle_through_the_list_when_the_plugin_cannot_be_read_any_more', async () => {
    commands.list.mockResolvedValue([plugin('a')]);
    await queryClient.fetchQuery(pluginsQueries.list());
    commands.get.mockRejectedValue(new Error('gone'));

    await pluginsCache.refresh.plugin(queryClient, 'a');

    expect(queryClient.getQueryData(pluginKeys.detail('a'))).toBeUndefined();
    // Nothing observes the list here, so it is marked for the next read.
    expect(queryClient.getQueryState(pluginKeys.list())!.isInvalidated).toBe(
      true,
    );
  });
});
