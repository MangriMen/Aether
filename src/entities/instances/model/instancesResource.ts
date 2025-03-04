import type { Accessor } from 'solid-js';
import { createMemo, createSignal } from 'solid-js';

import type { Instance } from '@/entities/instances';
import { getInstance, listInstances } from '@/entities/instances';
import { ReactiveMap } from '@solid-primitives/map';

const [isMappedInstancesLoading, setIsMappedInstancesLoading] =
  createSignal(false);

const fetchInstances = async () => {
  try {
    setIsMappedInstancesLoading(true);
    const [instances, _] = await listInstances();

    for (const instance of instances) {
      mappedInstances.set(instance.id, instance);
    }
  } catch (e) {
    console.error("Can't get minecraft instances", e);
  }

  setIsMappedInstancesLoading(false);
};

const fetchInstance = async (id: Instance['id']) => {
  try {
    const instance = await getInstance(id);

    mappedInstances.set(instance.id, instance);
  } catch (e) {
    console.error("Can't get minecraft instance", e);
    mappedInstances.delete(id);
  }
};

export const initializeInstances = fetchInstances;

const mappedInstances = new ReactiveMap<string, Instance>();

export const useInstances = (): [
  typeof mappedInstances,
  { refetch: () => void; isLoading: Accessor<boolean> },
] => [
  mappedInstances,
  { refetch: refetchInstances, isLoading: isMappedInstancesLoading },
];

export const refetchInstances = fetchInstances;
export const refetchInstance = fetchInstance;

const getCachedInstance = (id: Instance['id']) => mappedInstances.get(id);

export const useInstance = (id: Accessor<Instance['id']>) => {
  const memoizedInstance = createMemo(() => getCachedInstance(id()));
  return memoizedInstance;
};
