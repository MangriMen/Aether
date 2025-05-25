import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
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
import { QUERY_KEYS } from './query_keys';
import type { Accessor } from 'solid-js';

export const useSyncPlugins = () =>
  useMutation(() => ({
    mutationFn: syncPluginsRaw,
  }));

export const usePlugins = () =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.PLUGIN.LIST(),
    queryFn: listPluginsRaw,
  }));

export const usePlugin = (id: Accessor<string>) =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.PLUGIN.GET(id()),
    queryFn: () => getPluginRaw(id()),
    enabled: !!id(),
  }));

export const usePluginSettings = (id: Accessor<string>) =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.PLUGIN.SETTINGS(id()),
    queryFn: () => getPluginSettingsRaw(id()),
    enabled: !!id(),
  }));

export const useGetPluginEnabled = (id: Accessor<string>) =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.PLUGIN.ENABLED(id()),
    queryFn: () => getIsPluginEnabledRaw(id()),
    enabled: !!id(),
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
