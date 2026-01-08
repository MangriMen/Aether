import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { type Accessor } from 'solid-js';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';
import { showToast } from '@/shared/ui';

import type {
  ContentRequest,
  InstallContentPayload,
  ContentType,
} from '../../model';

import {
  listContentRaw,
  disableContentRaw,
  enableContentRaw,
  removeContentsRaw,
  getContentProvidersRaw,
  getContentByProviderRaw,
  installContentRaw,
  getMetadataFieldToCheckInstalledRaw,
  importContentsRaw,
} from '../tauriApi';
import { invalidateInstanceContent } from './cache';
import { CONTENT_QUERY_KEYS } from './contentQueryKeys';

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

export const useInstanceContents = (id: Accessor<string>) => {
  return useQuery(() => ({
    queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id()),
    queryFn: () => listContentRaw(id()),
    enabled: !!id(),
  }));
};

export const useDisableContents = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ id, paths }: { id: string; paths: string[] }) =>
      disableContentRaw(id, paths),
    onSuccess: (_, { id }) => {
      invalidateInstanceContent(queryClient, id);
    },
    onError: (err) => {
      showError({
        title: t('content.failedToDisable'),
        err,
        t,
      });
    },
  }));
};

export const useEnableContents = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ id, paths }: { id: string; paths: string[] }) =>
      enableContentRaw(id, paths),
    onSuccess: (_, { id }) => {
      invalidateInstanceContent(queryClient, id);
    },
    onError: (err) => {
      showError({
        title: t('content.failedToEnable'),
        err,
        t,
      });
    },
  }));
};

export const useRemoveContents = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ id, paths }: { id: string; paths: string[] }) =>
      removeContentsRaw(id, paths),
    onSuccess: (_, { id }) => {
      invalidateInstanceContent(queryClient, id);
    },
    onError: (err) => {
      showError({
        title: t('content.failedToRemove'),
        err,
        t,
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
    onError: (err) => {
      showError({
        title: t('content.failedToInstall'),
        err,
        t,
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
    onError: (err) => {
      showError({
        title: t('content.failedToImport'),
        err,
        t,
      });
    },
  }));
};
