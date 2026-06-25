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

export const useForceEnablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: pluginsCommands.forceEnable,
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

// ── GitHub / Remote source hooks ──

export const usePluginSource = (id: Accessor<PluginId>) =>
  useQuery(() => pluginsQueries.source(id()));

export const useCheckForUpdates = (id: Accessor<PluginId>) =>
  useQuery(() => pluginsQueries.updates(id()));

export const usePluginPreview = (url: Accessor<string>) =>
  useQuery(() => pluginsQueries.preview(url()));

export const useInstallFromGithub = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      owner,
      repo,
      tag,
    }: {
      owner: string;
      repo: string;
      tag: string;
    }) => pluginsCommands.installFromGithub(owner, repo, tag),
    onSuccess: () => pluginsCache.invalidate.all(queryClient),
    onError: (err) => {
      showError({
        title: t('plugins.installFromGithubError'),
        err,
        t,
      });
    },
  }));
};

export const useUpdatePlugin = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, targetTag }: { id: string; targetTag?: string }) =>
      pluginsCommands.updatePlugin(id, targetTag ?? null),
    onSuccess: (_, { id }) => pluginsCache.invalidate.full(queryClient, id),
    onError: (err, { id }) => {
      const plugin = pluginsCache.getData.fromList(queryClient, id);

      const name =
        plugin?.manifest.metadata.name ??
        plugin?.manifest.metadata.id ??
        '"unknown"';

      showError({
        title: t('plugins.failedToUpdatePlugin', {
          name,
        }),
        err,
        t,
      });
    },
  }));
};

export const useInstallPluginFromProvider = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      identifier,
      downloadUrl,
      tagName,
      version,
    }: {
      identifier: string;
      downloadUrl: string;
      tagName: string;
      version: string;
    }) =>
      pluginsCommands.installPluginFromProvider(
        'git_hub',
        identifier,
        downloadUrl,
        tagName,
        version,
      ),
    onSuccess: () => pluginsCache.invalidate.all(queryClient),
    onError: (err) => {
      showError({
        title: t('plugins.installFromGithubError'),
        err,
        t,
      });
    },
  }));
};
