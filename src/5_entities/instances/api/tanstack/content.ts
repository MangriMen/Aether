import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import {
  getInstanceContentsRaw,
  disableInstanceContentsRaw,
  enableInstanceContentsRaw,
  removeInstanceContentRaw,
  removeInstanceContentsRaw,
  getContentProvidersRaw,
  getContentByProviderRaw,
  installContentRaw,
  getMetadataFieldToCheckInstalledRaw,
  importContentsRaw,
} from '../rawApi';
import type {
  ContentRequest,
  InstallContentPayload,
  ContentType,
} from '../../model';
import { showToast } from '@/shared/ui';
import { type Accessor } from 'solid-js';
import { useTranslation } from '@/shared/model';
import { CONTENT_QUERY_KEYS } from './content_query_keys';

export const useInstanceContents = (id: Accessor<string>) => {
  return useQuery(() => ({
    queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id()),
    queryFn: () => getInstanceContentsRaw(id()),
    enabled: !!id,
    reconcile: (oldData, newData) => ({ ...oldData, ...newData }),
  }));
};

export const useContentProviders = () => {
  return useQuery(() => ({
    queryKey: CONTENT_QUERY_KEYS.PROVIDERS(),
    queryFn: getContentProvidersRaw,
  }));
};

export const useContentByProvider = (
  payload: Accessor<ContentRequest | undefined>,
) => {
  return useQuery(() => ({
    queryKey: [
      ...CONTENT_QUERY_KEYS.BY_PROVIDER(payload()?.provider ?? ''),
      payload(),
    ],
    queryFn: () => getContentByProviderRaw(payload()!),
    enabled: !!payload(),
  }));
};

export const useMetadataFieldToCheckInstalled = (
  provider: Accessor<string | undefined>,
) => {
  return useQuery(() => ({
    queryKey: CONTENT_QUERY_KEYS.METADATA_FIELD(provider()!),
    queryFn: () => getMetadataFieldToCheckInstalledRaw(provider()!),
    enabled: !!provider(),
  }));
};

export const useDisableContents = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, paths }: { id: string; paths: string[] }) =>
      disableInstanceContentsRaw(id, paths),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id),
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
        queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id),
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
        queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id),
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
        queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id),
      });
    },
  }));
};

export const useInstallContent = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({
      id,
      payload,
    }: {
      id: string;
      payload: InstallContentPayload;
    }) => installContentRaw(id, payload),
    onSuccess: (_, { id, payload }) => {
      queryClient.invalidateQueries({
        queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id),
      });
      showToast({
        title: t('content.installed', {
          contentType: t(`content.${payload.contentType}`) || '',
        }),
        variant: 'success',
      });
    },
  }));
};

export const useImportContents = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

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
    onSuccess: (_, { id, type }) => {
      queryClient.invalidateQueries({
        queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id),
      });
      showToast({
        title: t('content.imported', {
          contentType: t(`content.${type}`) || '',
        }),
        variant: 'success',
      });
    },
  }));
};
