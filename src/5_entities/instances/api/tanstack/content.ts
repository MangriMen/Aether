import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/solid-query';
import { type Accessor } from 'solid-js';

import type { PartialBy } from '@/shared/model';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';
import { showToast } from '@/shared/ui';

import type { ContentSearchParams, Instance } from '../../model';
import type { ContentCompatibilityCheckParams } from '../../model/compatibility';

import { ContentType } from '../../model';
import {
  listContentRaw,
  disableContentRaw,
  enableContentRaw,
  removeContentsRaw,
  listContentProvidersRaw,
  searchContentRaw,
  installContentRaw,
  importContentsRaw,
  checkCompatibility,
} from '../tauriApi';
import { invalidateInstanceContent } from './cache';
import { CONTENT_QUERY_KEYS } from './contentQueryKeys';

export const useContentProviders = () => {
  return useQuery(() => ({
    queryKey: CONTENT_QUERY_KEYS.PROVIDERS(),
    queryFn: listContentProvidersRaw,
  }));
};

export const useSearchContent = (
  payload: Accessor<ContentSearchParams | undefined>,
) => {
  return useQuery(() => {
    const data = payload();

    return {
      queryKey: data
        ? CONTENT_QUERY_KEYS.SEARCH(data)
        : CONTENT_QUERY_KEYS.SEARCH_EMPTY(),
      queryFn: () => searchContentRaw(data!),
      enabled: Boolean(data),
      placeholderData: keepPreviousData,
    };
  });
};

export const instanceContentsQuery = (id: string | undefined) => ({
  queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id ?? ''),
  queryFn: () => listContentRaw(id ?? ''),
  enabled: !!id,
});

export const useInstanceContents = (id: Accessor<string | undefined>) => {
  return useQuery(() => instanceContentsQuery(id()));
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
    mutationFn: installContentRaw,
    onSuccess: (_, payload) => {
      if (payload.type === 'atomic') {
        invalidateInstanceContent(queryClient, payload.data.instanceId);
      }

      const contentType =
        payload.type === 'atomic'
          ? payload.data.contentType
          : ContentType.Modpack;

      showToast({
        title: t('content.installed', {
          contentType: t(`content.${contentType}`) || '',
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

export const CHECK_COMPATIBILITY_QUERY = (
  ids: Instance['id'][],
  params: PartialBy<ContentCompatibilityCheckParams, 'providerId'>,
) => {
  const isEnabled = Boolean(params.providerId) && ids.length > 0;

  return {
    queryKey: [
      'compatibility',
      ids,
      params.providerId,
      params.contentItem.id,
    ] as const,
    queryFn: () =>
      checkCompatibility(ids, params as ContentCompatibilityCheckParams),
    enabled: isEnabled,
    placeholderData: keepPreviousData,
  };
};

export const useCheckCompatibility = (
  instanceIds: Accessor<Instance['id'][]>,
  checkParams: Accessor<
    PartialBy<ContentCompatibilityCheckParams, 'providerId'>
  >,
) => useQuery(() => CHECK_COMPATIBILITY_QUERY(instanceIds(), checkParams()));
