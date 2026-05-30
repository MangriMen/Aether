import type { Accessor } from 'solid-js';

import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { EditPluginSettingsDto } from '../api';
import type { PluginId } from './pluginManifest';

import { pluginsCommands } from '../api';
import { pluginsCache, pluginsQueries } from './cache';
import { pluginKeys } from './queryKeys';

export const usePlugins = () => useQuery(pluginsQueries.list);

export const usePlugin = (id: Accessor<PluginId>) => {
  const queryClient = useQueryClient();

  return useQuery(() => pluginsQueries.get(queryClient, id()));
};

export const usePluginSettings = (id: Accessor<PluginId>) =>
  useQuery(() => pluginsQueries.settings(id()));

export const useApiVersion = () => useQuery(pluginsQueries.apiVersion);

export const useSyncPlugins = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: pluginsCommands.sync,
    onSuccess: () => pluginsCache.invalidate.all(queryClient),
    onError: (err) => {
      showError({
        title: t('plugins.syncError'),
        err,
        t,
      });
    },
  }));
};

export const useEnablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: pluginsCommands.enable,
    onSuccess: (_, id) => pluginsCache.invalidate.full(queryClient, id),
  }));
};

export const useDisablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: pluginsCommands.disable,
    onSuccess: (_, id) => pluginsCache.invalidate.full(queryClient, id),
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
    }) => pluginsCommands.editSettings(id, editSettings),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: pluginKeys.settings(variables.id),
      });
    },
    onError: (err, variables) => {
      const plugin = pluginsCache.getData.fromList(queryClient, variables.id);

      const name =
        plugin?.manifest.metadata.name ??
        plugin?.manifest.metadata.id ??
        '"unknown"';

      showError({
        title: t('pluginSettings.failedToUpdateSettings', {
          name,
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
    mutationFn: pluginsCommands.openPluginsFolder,
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
    mutationFn: pluginsCommands.import,
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
    mutationFn: pluginsCommands.remove,
    onSuccess: (_, id) => pluginsCache.invalidate.full(queryClient, id),
    onError: (err, id) => {
      const plugin = pluginsCache.getData.fromList(queryClient, id);

      const name =
        plugin?.manifest.metadata.name ??
        plugin?.manifest.metadata.id ??
        '"unknown"';

      showError({
        title: t('plugins.failedToRemove', {
          name,
        }),
        err,
        t,
      });
    },
  }));
};
