import { createResource, createSignal } from 'solid-js';

// eslint-disable-next-line boundaries/element-types
import { getMinecraftInstances, Instance } from '@/entities/minecraft';

const mapInstancesToIds = (
  instances: Instance[] | undefined,
): Record<string, Instance> | undefined => {
  return instances?.reduce<Record<string, Instance>>((acc, instance) => {
    acc[instance.id] = instance;
    return acc;
  }, {});
};

const instancesResource = createResource(
  () => {
    try {
      const res = getMinecraftInstances();
      res.then((instances) =>
        setMappedInstances(mapInstancesToIds(instances?.[0])),
      );

      return res;
    } catch {
      console.error("Can't get minecraft instances");
    }
  },
  {
    initialValue: [[], []],
  },
);

const [mappedInstances, setMappedInstances] = createSignal<
  Record<string, Instance> | undefined
>();

export const useInstances = () => {
  return instancesResource[0];
};

export const refetchInstances = () => {
  instancesResource[1].refetch();
};

export const useMappedInstances = () => mappedInstances;
