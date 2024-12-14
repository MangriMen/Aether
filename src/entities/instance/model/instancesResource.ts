import { createResource } from 'solid-js';

// eslint-disable-next-line boundaries/element-types
import { getMinecraftInstances } from '@/entities/minecraft';

const instancesResource = createResource(
  () => {
    try {
      return getMinecraftInstances();
    } catch {
      console.error("Can't get minecraft instances");
    }
  },
  {
    initialValue: [[], []],
  },
);

export const getInstances = () => {
  return instancesResource[0];
};

export const refetchInstances = () => {
  instancesResource[1].refetch();
};
