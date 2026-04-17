import type { QueryClient } from '@tanstack/solid-query';
import type { Accessor } from 'solid-js';

import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { Plugin } from '.';
import type { EditPluginSettingsDto } from '../api';

import { commands } from '../api';
import {
  invalidateImporters,
  invalidatePluginData,
  invalidatePluginsData,
} from './cache';
import { pluginKeys } from './queryKeys';

export const useSyncPlugins = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: commands.sync,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: pluginKeys.list() });
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
    queryKey: pluginKeys.list(),
    queryFn: commands.list,
  }));

export const usePlugin = (id: Accessor<string>) =>
  useQuery(() => ({
    queryKey: pluginKeys.get(id()),
    queryFn: () => commands.get(id()),
    enabled: !!id(),
  }));

const pluginSettingsQuery = (id: Accessor<string>) => ({
  queryKey: pluginKeys.settings(id()),
  queryFn: () => commands.getSettings(id()),
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
    mutationFn: commands.enable,
    onSuccess: () => {
      invalidatePluginsData(queryClient);
      invalidateImporters(queryClient);
    },
  }));
};

export const useDisablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: commands.disable,
    onSuccess: () => {
      invalidatePluginsData(queryClient);
      invalidateImporters(queryClient);
    },
  }));
};

export const useEditPluginSettings = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      id,
      editSettings,
    }: {
      id: string;
      editSettings: EditPluginSettingsDto;
    }) => commands.editSettings(id, editSettings),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: pluginKeys.settings(variables.id),
      });
    },
    onError: (err, variables) => {
      // TODO: rewrite caching logic to get query data for plugin
      // instead of finding in list
      const plugin = queryClient
        .getQueryData<Plugin[]>(pluginKeys.list())
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
    mutationFn: commands.openPluginsFolder,
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
    mutationFn: commands.import,
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
    mutationFn: commands.remove,
    onSuccess: (_, id) => {
      invalidatePluginsData(queryClient);
      invalidatePluginData(queryClient, id);
      invalidateImporters(queryClient);
    },
    onError: (err, id) => {
      // TODO: rewrite caching logic to get query data for plugin
      // instead of finding in list
      const plugin = queryClient
        .getQueryData<Plugin[]>(pluginKeys.list())
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

export const useApiVersion = () => {
  return useQuery(() => ({
    queryKey: pluginKeys.apiVersion(),
    queryFn: commands.getApiVersion,
  }));
};
