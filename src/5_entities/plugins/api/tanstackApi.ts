import type { QueryClient } from '@tanstack/solid-query';
import type { Accessor } from 'solid-js';

import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import type { Plugin } from '../model';

import { PLUGIN_QUERY_KEYS } from './queryKeys';
import {
  disablePluginRaw,
  editPluginSettingsRaw,
  enablePluginRaw,
  getPluginRaw,
  getPluginSettingsRaw,
  importPluginsRaw,
  listPluginsRaw,
  openPluginsFolderRaw,
  removePluginRaw,
  syncPluginsRaw,
} from './tauriApi';

export const useSyncPlugins = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: syncPluginsRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: PLUGIN_QUERY_KEYS.LIST() });
    },
    onError: (err) => {
      showError({
        title: t('plugins.syncError'),
        err,
        t,
      });
    },
  }));
};

export const usePlugins = () =>
  useQuery(() => ({
    queryKey: PLUGIN_QUERY_KEYS.LIST(),
    queryFn: listPluginsRaw,
  }));

export const usePlugin = (id: Accessor<string>) =>
  useQuery(() => ({
    queryKey: PLUGIN_QUERY_KEYS.GET(id()),
    queryFn: () => getPluginRaw(id()),
    enabled: !!id(),
  }));

const pluginSettingsQuery = (id: Accessor<string>) => ({
  queryKey: PLUGIN_QUERY_KEYS.SETTINGS(id()),
  queryFn: () => getPluginSettingsRaw(id()),
  enabled: !!id(),
});

export const usePluginSettings = (id: Accessor<string>) =>
  useQuery(() => pluginSettingsQuery(id));

export const prefetchPluginSettings = (
  queryClient: QueryClient,
  id: Accessor<string>,
) => queryClient.prefetchQuery(pluginSettingsQuery(id));

export const useEnablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: enablePluginRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: PLUGIN_QUERY_KEYS.LIST() });
    },
  }));
};

export const useDisablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: disablePluginRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: PLUGIN_QUERY_KEYS.LIST() });
    },
  }));
};

export const useEditPluginSettings = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: editPluginSettingsRaw,
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: PLUGIN_QUERY_KEYS.SETTINGS(variables.id),
      });
    },
    onError: (err, variables) => {
      // TODO: rewrite caching logic to get query data for plugin
      // instead of finding in list
      const plugin = queryClient
        .getQueryData<Plugin[]>(PLUGIN_QUERY_KEYS.LIST())
        ?.find((plugin) => plugin.manifest.metadata.id === variables.id);

      showError({
        title: t('pluginSettings.failedToUpdateSettings', {
          name:
            plugin?.manifest.metadata.name ??
            plugin?.manifest.metadata.id ??
            '"unknown"',
        }),
        err,
        t,
      });
    },
  }));
};

export const useOpenPluginFolder = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: openPluginsFolderRaw,
    onError: (err) => {
      showError({
        title: t('plugins.openPluginsFolderError'),
        err,
        t,
      });
    },
  }));
};

export const useImportPlugins = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: importPluginsRaw,
    onError: (err) => {
      showError({
        title: t('plugins.failedToImport'),
        err,
        t,
      });
    },
  }));
};

export const useRemovePlugin = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: removePluginRaw,
    onError: (err, id) => {
      // TODO: rewrite caching logic to get query data for plugin
      // instead of finding in list
      const plugin = queryClient
        .getQueryData<Plugin[]>(PLUGIN_QUERY_KEYS.LIST())
        ?.find((plugin) => plugin.manifest.metadata.id === id);

      showError({
        title: t('plugins.failedToRemove', {
          name:
            plugin?.manifest.metadata.name ??
            plugin?.manifest.metadata.id ??
            '"unknown"',
        }),
        err,
        t,
      });
    },
  }));
};
