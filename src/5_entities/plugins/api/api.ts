import type { Accessor } from 'solid-js';

import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { QUERY_KEYS } from './query_keys';
import {
  disablePluginRaw,
  editPluginSettingsRaw,
  enablePluginRaw,
  getIsPluginEnabledRaw,
  getPluginRaw,
  getPluginSettingsRaw,
  listPluginsRaw,
  openPluginsFolderRaw,
  syncPluginsRaw,
} from './rawApi';

export const useSyncPlugins = () =>
  useMutation(() => ({
    mutationFn: syncPluginsRaw,
  }));

export const usePlugins = () =>
  useQuery(() => ({
    queryFn: listPluginsRaw,
    queryKey: QUERY_KEYS.PLUGIN.LIST(),
  }));

export const usePlugin = (id: Accessor<string>) =>
  useQuery(() => ({
    enabled: !!id(),
    queryFn: () => getPluginRaw(id()),
    queryKey: QUERY_KEYS.PLUGIN.GET(id()),
  }));

export const usePluginSettings = (id: Accessor<string>) =>
  useQuery(() => ({
    enabled: !!id(),
    queryFn: () => getPluginSettingsRaw(id()),
    queryKey: QUERY_KEYS.PLUGIN.SETTINGS(id()),
  }));

export const useGetPluginEnabled = (id: Accessor<string>) =>
  useQuery(() => ({
    enabled: !!id(),
    queryFn: () => getIsPluginEnabledRaw(id()),
    queryKey: QUERY_KEYS.PLUGIN.ENABLED(id()),
  }));

export const useEnablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: enablePluginRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.PLUGIN.LIST() });
    },
  }));
};

export const useDisablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: disablePluginRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.PLUGIN.LIST() });
    },
  }));
};

export const useEditPluginSettings = () => {
  const queryClient = useQueryClient();
  return useMutation(() => ({
    mutationFn: editPluginSettingsRaw,
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: QUERY_KEYS.PLUGIN.SETTINGS(variables.id),
      });
    },
  }));
};

export const useOpenPluginFolder = () =>
  useMutation(() => ({
    mutationFn: openPluginsFolderRaw,
  }));
