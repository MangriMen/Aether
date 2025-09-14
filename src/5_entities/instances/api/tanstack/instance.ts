import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { type Accessor, createMemo } from 'solid-js';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import type { EditInstance, InstanceImportDto } from '../../model';

import {
  createInstanceRaw,
  editInstanceRaw,
  getInstanceDirRaw,
  getInstanceRaw,
  importInstanceRaw,
  listInstancesRaw,
  removeInstanceRaw,
} from '../rawApi';
import { invalidateInstanceData } from './cache';
import { INSTANCE_QUERY_KEYS } from './instance_query_keys';

const INSTANCE_RECONCILE = 'id';

export const useCreateInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: createInstanceRaw,
    onError: (err) => {
      showError({
        err,
        t,
        title: t('instance.createError'),
      });
    },
  }));
};

export const useImportInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (dto: InstanceImportDto) => importInstanceRaw(dto),
    onError: (err) => {
      showError({
        err,
        t,
        title: t('instance.importError'),
      });
    },
  }));
};

export const useInstances = () => {
  return useQuery(() => ({
    queryFn: listInstancesRaw,
    queryKey: INSTANCE_QUERY_KEYS.LIST(),
    reconcile: INSTANCE_RECONCILE,
  }));
};

export const useInstance = (id: Accessor<string>) => {
  const instances = useInstances();

  const instanceFromCache = createMemo(() =>
    instances.data?.find((instance) => instance.id === id()),
  );

  const shouldFetch = createMemo(
    () => !instanceFromCache() && !instances.isLoading && !!id(),
  );

  // Try to fetch instance if not found in cache
  const instanceQuery = useQuery(() => ({
    enabled: shouldFetch(),
    queryFn: () => getInstanceRaw(id()),
    queryKey: INSTANCE_QUERY_KEYS.GET(id()),
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
    get isError() {
      return isError();
    },
    get isFetching() {
      return instances.isFetching || instanceQuery.isFetching;
    },
    get isLoading() {
      return isLoading();
    },
  };
};

export const useEditInstance = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ edit, id }: { edit: EditInstance; id: string }) =>
      editInstanceRaw(id, edit),
    onError: (err, values) => {
      showError({
        err,
        t,
        title: t('instance.editError', { id: values.id }),
      });
    },
    onSuccess: (_, { id }) => {
      invalidateInstanceData(queryClient, id);
    },
  }));
};

export const useRemoveInstance = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: (id: string) => removeInstanceRaw(id),
    onError: (err, id) => {
      showError({
        err,
        t,
        title: t('instance.removeError', { id }),
      });
    },
    onSuccess: (_, id) => {
      invalidateInstanceData(queryClient, id);
    },
  }));
};

export const useInstanceDir = (id: Accessor<string>) => {
  return useQuery(() => ({
    enabled: !!id(),
    queryFn: () => getInstanceDirRaw(id()),
    queryKey: [...INSTANCE_QUERY_KEYS.DIR(id())],
  }));
};
