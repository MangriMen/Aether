import { Accessor, createResource } from 'solid-js';

import {
  getMinecraftInstanceProcess,
  getRunningMinecraftInstances,
  // eslint-disable-next-line boundaries/element-types
} from '@/entities/minecraft';

const runningInstancesData = createResource(() => {
  try {
    return getRunningMinecraftInstances();
  } catch {
    console.error("Can't get running instances");
  }
});

export const useRunningInstancesData = () => {
  return runningInstancesData[0];
};

export const refetchRunningInstancesData = () => {
  return runningInstancesData[1].refetch();
};

export const createRunningInstanceDataResource = (id: Accessor<string>) =>
  createResource(id, (id) => {
    try {
      return getMinecraftInstanceProcess(id);
    } catch {
      console.error("Can't get running instance");
    }
  });
