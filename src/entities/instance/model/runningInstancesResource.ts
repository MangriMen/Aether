import { Accessor, createResource } from 'solid-js';

import {
  getMinecraftInstanceProcess,
  getRunningMinecraftInstances,
  // eslint-disable-next-line boundaries/element-types
} from '@/entities/minecraft';

const runningInstances = createResource(() => {
  try {
    return getRunningMinecraftInstances();
  } catch {
    console.error("Can't get running instances");
  }
});

export const getRunningInstances = () => {
  return runningInstances[0];
};

export const refetchRunningInstances = () => {
  return runningInstances[1].refetch();
};

export const createRunningInstanceResource = (id: Accessor<string>) =>
  createResource(id, (id) => {
    try {
      return getMinecraftInstanceProcess(id);
    } catch {
      console.error("Can't get running instance");
    }
  });
