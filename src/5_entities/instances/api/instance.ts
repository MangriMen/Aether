import type { QueryClient } from '@tanstack/solid-query';
import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import {
  createInstanceRaw,
  installInstanceRaw,
  updateInstanceRaw,
  importInstanceRaw,
  listInstancesRaw,
  getInstanceRaw,
  launchInstanceRaw,
  stopInstanceRaw,
  removeInstanceRaw,
  editInstanceRaw,
  getInstanceDirRaw,
} from './rawApi';
import type { Instance, InstanceImportDto, EditInstance } from '../model';
import { showToast } from '@/shared/ui';
import { QUERY_KEYS } from './query_keys';
import type { Accessor } from 'solid-js';

export const useInstances = () => {
  return useQuery(() => ({
    queryKey: QUERY_KEYS.INSTANCE.LIST(),
    queryFn: listInstancesRaw,
  }));
};

export const useInstance = (id: Accessor<string>) => {
  const queryClient = useQueryClient();
  const instancesQuery = useInstances();

  // Получаем экземпляр из общего списка
  const getSharedInstance = () => {
    const instances = queryClient.getQueryData<Instance[]>(
      QUERY_KEYS.INSTANCE.LIST(),
    );
    return instances?.find((inst) => inst.id === id());
  };

  // Запрос деталей экземпляра (только если нет в общем списке)
  const detailQuery = useQuery(() => ({
    queryKey: QUERY_KEYS.INSTANCE.GET(id()),
    queryFn: () => getInstanceRaw(id()),
    initialData: getSharedInstance,
    initialDataUpdatedAt: () =>
      queryClient.getQueryState(QUERY_KEYS.INSTANCE.LIST())?.dataUpdatedAt,
    enabled: !getSharedInstance() && !!id(),
  }));

  return {
    data: getSharedInstance() || detailQuery.data,
    isLoading: instancesQuery.isLoading || detailQuery.isLoading,
    isError: instancesQuery.isError || detailQuery.isError,
    error: instancesQuery.error || detailQuery.error,
    refetch: detailQuery.refetch,
  };
};

export const useInstanceDir = (id: Accessor<string>) => {
  return useQuery(() => ({
    queryKey: [...QUERY_KEYS.INSTANCE.GET(id()), 'dir'],
    queryFn: () => getInstanceDirRaw(id()),
    enabled: !!id(),
  }));
};

export const useCreateInstance = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: createInstanceRaw,
    onSuccess: (newInstance) => {
      // Оптимистично добавляем в список
      queryClient.setQueryData(
        QUERY_KEYS.INSTANCE.LIST(),
        (old: Instance[] = []) => [...old, newInstance],
      );
      showToast({
        title: 'Instance created',
        variant: 'success',
      });
    },
  }));
};

export const useInstallInstance = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, force }: { id: string; force?: boolean }) =>
      installInstanceRaw(id, force ?? false),
    onSuccess: (_, { id }) => {
      // Инвалидируем только конкретный экземпляр
      queryClient.invalidateQueries({
        queryKey: QUERY_KEYS.INSTANCE.GET(id),
        refetchType: 'none', // Только помечаем как устаревшие
      });
      showToast({
        title: 'Instance installed',
        variant: 'success',
      });
    },
  }));
};

export const useUpdateInstance = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: (id: string) => updateInstanceRaw(id),
    onSuccess: (_, id) => {
      // Обновляем в списке
      // updateInstanceCacheData(queryClient, updatedInstance, id);
      invalidateInstanceData(queryClient, id);
      showToast({
        title: 'Instance updated',
        variant: 'success',
      });
    },
  }));
};

export const useImportInstance = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: (dto: InstanceImportDto) => importInstanceRaw(dto),
    onSuccess: (newInstance) => {
      // Добавляем новый экземпляр в список
      queryClient.setQueryData(
        QUERY_KEYS.INSTANCE.LIST(),
        (old: Instance[] = []) => [...old, newInstance],
      );
      showToast({
        title: 'Instance imported',
        variant: 'success',
      });
    },
  }));
};

export const useLaunchInstance = () => {
  return useMutation(() => ({
    mutationFn: (id: string) => launchInstanceRaw(id),
    onSuccess: () => {
      showToast({
        title: 'Instance launched',
        variant: 'success',
      });
    },
  }));
};

export const useStopInstance = () => {
  return useMutation(() => ({
    mutationFn: (uuid: string) => stopInstanceRaw(uuid),
    onSuccess: () => {
      showToast({
        title: 'Instance stopped',
        variant: 'success',
      });
    },
  }));
};

export const useRemoveInstance = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: (id: string) => removeInstanceRaw(id),
    onSuccess: (_, id) => {
      // Удаляем из списка и детали
      removeInstanceData(queryClient, id);
      showToast({
        title: 'Instance removed',
        variant: 'success',
      });
    },
  }));
};

export const useEditInstance = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, edit }: { id: string; edit: EditInstance }) =>
      editInstanceRaw(id, edit),
    onMutate: async ({ id, edit }) => {
      // Отменяем текущие запросы
      await queryClient.cancelQueries({
        queryKey: QUERY_KEYS.INSTANCE.GET(id),
      });

      // Сохраняем предыдущее значение
      const prevInstance = queryClient.getQueryData(
        QUERY_KEYS.INSTANCE.GET(id),
      );

      // Оптимистичное обновление
      queryClient.setQueryData(
        QUERY_KEYS.INSTANCE.GET(id),
        (old: Instance | undefined) => (old ? { ...old, ...edit } : undefined),
      );

      return { prevInstance };
    },
    onError: (_err, { id }, context) => {
      if (context?.prevInstance) {
        queryClient.setQueryData(
          QUERY_KEYS.INSTANCE.GET(id),
          context.prevInstance,
        );
      }
    },
    onSuccess: (_, { id }) => {
      // updateInstanceCacheData(queryClient, updatedInstance, id);
      invalidateInstanceData(queryClient, id);
      showToast({
        title: 'Instance updated',
        variant: 'success',
      });
    },
  }));
};

export const updateInstanceCacheData = (
  queryClient: QueryClient,
  updatedInstance: Instance,
  id: Instance['id'],
) => {
  queryClient.setQueryData(QUERY_KEYS.INSTANCE.LIST(), (old: Instance[] = []) =>
    old.map((inst) => (inst.id === id ? updatedInstance : inst)),
  );
  queryClient.setQueryData(QUERY_KEYS.INSTANCE.GET(id), updatedInstance);
};

export const invalidateInstanceData = (
  queryClient: QueryClient,
  id: Instance['id'],
) => {
  queryClient.invalidateQueries({
    queryKey: QUERY_KEYS.INSTANCE.GET(id),
  });
  queryClient.invalidateQueries({
    queryKey: QUERY_KEYS.CONTENT.BY_INSTANCE(id),
  });
};

export const removeInstanceData = (
  queryClient: QueryClient,
  id: Instance['id'],
) => {
  queryClient.setQueryData(QUERY_KEYS.INSTANCE.LIST(), (old: Instance[] = []) =>
    old.filter((inst) => inst.id !== id),
  );
  queryClient.removeQueries({ queryKey: QUERY_KEYS.INSTANCE.GET(id) });
};
