import type { Accessor, InitializedResourceReturn } from 'solid-js';
import { createMemo, createResource, createSignal } from 'solid-js';

import type { Instance } from '@/entities/instances';
import { getMinecraftInstances } from '@/entities/instances';

const mapInstancesToIds = (
  instances: Instance[] | undefined,
): Record<string, Instance> | undefined => {
  return instances?.reduce<Record<string, Instance>>((acc, instance) => {
    acc[instance.id] = instance;
    return acc;
  }, {});
};

let instancesResource:
  | InitializedResourceReturn<[Instance[], string[]] | undefined, unknown>
  | undefined;

export const initializeInstanceResource = () => {
  instancesResource = createResource(
    () => {
      try {
        const res = getMinecraftInstances();
        res.then((instances) =>
          setMappedInstances(mapInstancesToIds(instances?.[0])),
        );

        return res;
      } catch (e) {
        console.error("Can't get minecraft instances", e);
      }
    },
    {
      initialValue: [[], []],
    },
  );
};

const [mappedInstances, setMappedInstances] = createSignal<
  Record<string, Instance> | undefined
>();

export const useInstances = () => {
  return instancesResource?.[0];
};

export const refetchInstances = () => {
  instancesResource?.[1].refetch();
};

export const useMappedInstances = () => mappedInstances;

const getCachedInstance = (id: Instance['id']) => mappedInstances()?.[id];

export const useInstance = (id: Accessor<Instance['id']>) => {
  const memoizedInstance = createMemo(() => getCachedInstance(id()));
  return memoizedInstance;
};
