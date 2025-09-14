import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { type Accessor } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { showToast } from '@/shared/ui';

import type {
  ContentRequest,
  ContentType,
  InstallContentPayload,
} from '../../model';

import {
  disableInstanceContentsRaw,
  enableInstanceContentsRaw,
  getContentByProviderRaw,
  getContentProvidersRaw,
  getInstanceContentsRaw,
  getMetadataFieldToCheckInstalledRaw,
  importContentsRaw,
  installContentRaw,
  removeInstanceContentRaw,
  removeInstanceContentsRaw,
} from '../rawApi';
import { invalidateInstanceContent } from './cache';
import { CONTENT_QUERY_KEYS } from './content_query_keys';

export const useContentProviders = () => {
  return useQuery(() => ({
    queryFn: getContentProvidersRaw,
    queryKey: CONTENT_QUERY_KEYS.PROVIDERS(),
  }));
};

export const useContentByProvider = (
  payload: Accessor<ContentRequest | undefined>,
) => {
  return useQuery(() => ({
    enabled: !!payload(),
    queryFn: () => getContentByProviderRaw(payload()!),
    queryKey: [
      ...CONTENT_QUERY_KEYS.BY_PROVIDER(payload()?.provider ?? ''),
      payload(),
    ],
  }));
};

export const useMetadataFieldToCheckInstalled = (
  provider: Accessor<string | undefined>,
) => {
  return useQuery(() => ({
    enabled: !!provider(),
    queryFn: () => getMetadataFieldToCheckInstalledRaw(provider()!),
    queryKey: CONTENT_QUERY_KEYS.METADATA_FIELD(provider()!),
  }));
};

export const useInstanceContents = (id: Accessor<string>) => {
  return useQuery(() => ({
    enabled: !!id(),
    queryFn: () => getInstanceContentsRaw(id()),
    queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id()),
  }));
};

export const useDisableContents = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, paths }: { id: string; paths: string[] }) =>
      disableInstanceContentsRaw(id, paths),
    onSuccess: (_, { id }) => {
      invalidateInstanceContent(queryClient, id);
    },
  }));
};

export const useEnableContents = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, paths }: { id: string; paths: string[] }) =>
      enableInstanceContentsRaw(id, paths),
    onSuccess: (_, { id }) => {
      invalidateInstanceContent(queryClient, id);
    },
  }));
};

export const useRemoveContent = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, path }: { id: string; path: string }) =>
      removeInstanceContentRaw(id, path),
    onSuccess: (_, { id }) => {
      invalidateInstanceContent(queryClient, id);
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
      invalidateInstanceContent(queryClient, id);

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
      invalidateInstanceContent(queryClient, id);

      showToast({
        title: t('content.imported', {
          contentType: t(`content.${type}`) || '',
        }),
        variant: 'success',
      });
    },
  }));
};
