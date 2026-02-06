import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { createMemo, type Accessor } from 'solid-js';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import type { ImportInstance, EditInstance } from '../../model';

import {
  createInstanceRaw,
  importInstanceRaw,
  listInstancesRaw,
  getInstanceRaw,
  removeInstanceRaw,
  editInstanceRaw,
  getInstanceDirRaw,
} from '../tauriApi';
import { invalidateInstanceData } from './cache';
import { INSTANCE_QUERY_KEYS } from './instanceQueryKeys';

const INSTANCE_RECONCILE = 'id';

export const useCreateInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: createInstanceRaw,
    onError: (err) => {
      showError({
        title: t('instance.createError'),
        err,
        t,
      });
    },
  }));
};

export const useImportInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (dto: ImportInstance) => importInstanceRaw(dto),
    onError: (err) => {
      showError({
        title: t('instance.importError'),
        err,
        t,
      });
    },
  }));
};

export const useInstances = () => {
  return useQuery(() => ({
    queryKey: INSTANCE_QUERY_KEYS.LIST(),
    queryFn: listInstancesRaw,
    reconcile: INSTANCE_RECONCILE,
  }));
};

export const useInstance = (id: Accessor<string | undefined>) => {
  const instances = useInstances();

  const instanceFromCache = createMemo(() =>
    instances.data?.find((instance) => instance.id === id()),
  );

  const shouldFetch = createMemo(
    () => !instanceFromCache() && !instances.isLoading && !!id(),
  );

  // Try to fetch instance if not found in cache
  const instanceQuery = useQuery(() => ({
    queryKey: INSTANCE_QUERY_KEYS.GET(id() ?? ''),
    queryFn: () => getInstanceRaw(id() ?? ''),
    enabled: shouldFetch(),
    reconcile: INSTANCE_RECONCILE,
  }));

  // Combine query data and save reactivity
  const data = createMemo(() => instanceFromCache() ?? instanceQuery.data);
  const isLoading = createMemo(
    () => instances.isLoading || (shouldFetch() && instanceQuery.isLoading),
  );
  const isError = createMemo(() => instances.isError || instanceQuery.isError);

  return {
    get data() {
      return data();
    },
    get isLoading() {
      return isLoading();
    },
    get isError() {
      return isError();
    },
    get isFetching() {
      return instances.isFetching || instanceQuery.isFetching;
    },
  };
};

export const useEditInstance = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, edit }: { id: string; edit: EditInstance }) =>
      editInstanceRaw(id, edit),
    onSuccess: (_, { id }) => {
      invalidateInstanceData(queryClient, id);
    },
    onError: (err, values) => {
      showError({
        title: t('instance.editError', { id: values.id }),
        err,
        t,
      });
    },
  }));
};

export const useRemoveInstance = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: (id: string) => removeInstanceRaw(id),
    onSuccess: (_, id) => {
      invalidateInstanceData(queryClient, id);
    },
    onError: (err, id) => {
      showError({
        title: t('instance.removeError', { id }),
        err,
        t,
      });
    },
  }));
};

export const useInstanceDir = (id: Accessor<string>) => {
  return useQuery(() => ({
    queryKey: [...INSTANCE_QUERY_KEYS.DIR(id())],
    queryFn: () => getInstanceDirRaw(id()),
    enabled: !!id(),
  }));
};
