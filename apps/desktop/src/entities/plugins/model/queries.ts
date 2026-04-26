import type { Accessor } from 'solid-js';

import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import type { EditPluginSettingsDto } from '../api';
import type { PluginId } from './pluginManifest';

import { showError } from '../../../shared/lib';
import { useTranslation } from '../../../shared/model';
import { commands } from '../api';
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
    mutationFn: commands.sync,
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
    mutationFn: commands.enable,
    onSuccess: (_, id) => pluginsCache.invalidate.full(queryClient, id),
  }));
};

export const useDisablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: commands.disable,
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
    }) => commands.editSettings(id, editSettings),
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
