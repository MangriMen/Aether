import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/solid-query';
import { type Accessor } from 'solid-js';

import type { PartialBy } from '@/shared/model';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { showToast } from '@/shared/ui';

import type {
  ContentGetParams,
  ContentListVersionParams,
  ContentSearchParams,
  Instance,
} from '..';
import type { ContentCompatibilityCheckParams } from '../compatibility';

import { ContentType } from '..';
import { commands } from '../../api';
import { invalidateInstanceContent } from './cache';
import { CONTENT_QUERY_KEYS } from './contentQueryKeys';

export const useContentProviders = () => {
  return useQuery(() => ({
    queryKey: CONTENT_QUERY_KEYS.PROVIDERS(),
    queryFn: commands.listContentProviders,
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
      queryFn: () => commands.searchContent(data!),
      enabled: Boolean(data),
      placeholderData: keepPreviousData,
    };
  });
};

export const instanceContentsQuery = (id: string | undefined) => ({
  queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id ?? ''),
  queryFn: () => commands.listContent(id ?? ''),
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
      commands.disableContents(id, paths),
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
      commands.enableContents(id, paths),
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
      commands.removeContents(id, paths),
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
    mutationFn: commands.installContent,
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
    }) => commands.importContents(id, type, paths),
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
    queryKey: CONTENT_QUERY_KEYS.COMPATIBILITY(
      ids,
      params as ContentCompatibilityCheckParams,
    ),
    queryFn: () =>
      commands.checkCompatibility(
        ids,
        params as ContentCompatibilityCheckParams,
      ),
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

export const useContent = (params: Accessor<ContentGetParams | undefined>) => {
  return useQuery(() => {
    const params_ = params();

    return {
      queryKey: params_
        ? CONTENT_QUERY_KEYS.GET(params_)
        : CONTENT_QUERY_KEYS.GET_EMPTY(),
      queryFn: () => commands.getContent(params_!),
      enabled: Boolean(params_),
    };
  });
};

export const useContentVersion = (
  params: Accessor<ContentListVersionParams | undefined>,
) => {
  return useQuery(() => {
    const params_ = params();

    return {
      queryKey: params_
        ? CONTENT_QUERY_KEYS.LIST_CONTENT_VERSION(params_)
        : CONTENT_QUERY_KEYS.LIST_CONTENT_VERSION_EMPTY(),
      queryFn: () => commands.listContentVersion(params_!),
      enabled: Boolean(params_),
    };
  });
};
