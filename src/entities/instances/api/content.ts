import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import {
  getInstanceContentsRaw,
  toggleDisableInstanceContentRaw,
  disableInstanceContentsRaw,
  enableInstanceContentsRaw,
  removeInstanceContentRaw,
  removeInstanceContentsRaw,
  getContentProvidersRaw,
  getContentByProviderRaw,
  installContentRaw,
  getMetadataFieldToCheckInstalledRaw,
  importContentsRaw,
} from './rawApi';
import type {
  ContentRequest,
  InstallContentPayload,
  ContentType,
} from '../model';
import { showToast } from '@/shared/ui';
import type { Accessor } from 'solid-js';

// Ключи для кэширования контента
export const contentKeys = {
  all: ['contents'] as const,
  lists: () => [...contentKeys.all, 'list'] as const,
  instanceContents: (id: string) =>
    [...contentKeys.all, 'instance', id] as const,
  providers: () => [...contentKeys.all, 'providers'] as const,
  providerContent: (provider: string) =>
    [...contentKeys.all, 'provider', provider] as const,
};

// Запросы данных
export const useInstanceContents = (id: Accessor<string>) => {
  return useQuery(() => ({
    queryKey: contentKeys.instanceContents(id()),
    queryFn: () => getInstanceContentsRaw(id()),
    enabled: !!id, // Только если ID существует,
  }));
};

export const useContentProviders = () => {
  return useQuery(() => ({
    queryKey: contentKeys.providers(),
    queryFn: getContentProvidersRaw,
  }));
};

export const useContentByProvider = (
  payload: Accessor<ContentRequest | undefined>,
) => {
  return useQuery(() => ({
    queryKey: [...contentKeys.providerContent(payload()!.provider), payload()],
    queryFn: () => getContentByProviderRaw(payload()!),
    enabled: !!payload(),
  }));
};

export const useMetadataFieldToCheckInstalled = (
  provider: Accessor<string | undefined>,
) => {
  return useQuery(() => ({
    queryKey: [...contentKeys.providerContent(provider()!), 'metadata-field'],
    queryFn: () => getMetadataFieldToCheckInstalledRaw(provider()!),
    enabled: !!provider(), // Только если провайдер существует
  }));
};

// Мутации
export const useToggleDisableContent = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, path }: { id: string; path: string }) =>
      toggleDisableInstanceContentRaw(id, path),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: contentKeys.instanceContents(id),
      });
    },
  }));
};

export const useDisableContents = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, paths }: { id: string; paths: string[] }) =>
      disableInstanceContentsRaw(id, paths),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: contentKeys.instanceContents(id),
      });
    },
  }));
};

export const useEnableContents = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, paths }: { id: string; paths: string[] }) =>
      enableInstanceContentsRaw(id, paths),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: contentKeys.instanceContents(id),
      });
    },
  }));
};

export const useRemoveContent = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, path }: { id: string; path: string }) =>
      removeInstanceContentRaw(id, path),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: contentKeys.instanceContents(id),
      });
    },
  }));
};

export const useRemoveContents = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, paths }: { id: string; paths: string[] }) =>
      removeInstanceContentsRaw(id, paths),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: contentKeys.instanceContents(id),
      });
    },
  }));
};

export const useInstallContent = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      id,
      payload,
    }: {
      id: string;
      payload: InstallContentPayload;
    }) => installContentRaw(id, payload),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: contentKeys.instanceContents(id),
      });
      showToast({
        title: 'Content installed',
        variant: 'success',
      });
    },
  }));
};

export const useImportContents = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      id,
      paths,
      type,
    }: {
      id: string;
      paths: string[];
      type: ContentType;
    }) => importContentsRaw(id, paths, type),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: contentKeys.instanceContents(id),
      });
      showToast({
        title: 'Contents imported',
        variant: 'success',
      });
    },
  }));
};
