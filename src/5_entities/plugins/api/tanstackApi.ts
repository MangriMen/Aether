import type { Accessor } from 'solid-js';

import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import { PLUGIN_QUERY_KEYS } from './query_keys';
import {
  disablePluginRaw,
  editPluginSettingsRaw,
  enablePluginRaw,
  getPluginRaw,
  getPluginSettingsRaw,
  listPluginsRaw,
  openPluginsFolderRaw,
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

export const usePluginSettings = (id: Accessor<string>) =>
  useQuery(() => ({
    queryKey: PLUGIN_QUERY_KEYS.SETTINGS(id()),
    queryFn: () => getPluginSettingsRaw(id()),
    enabled: !!id(),
  }));

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
  const queryClient = useQueryClient();
  return useMutation(() => ({
    mutationFn: editPluginSettingsRaw,
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: PLUGIN_QUERY_KEYS.SETTINGS(variables.id),
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
