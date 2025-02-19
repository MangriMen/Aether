import type { Accessor } from 'solid-js';
import { createResource } from 'solid-js';

import { getInstanceProcess, listProcess } from '../api';

const runningInstancesData = createResource(() => {
  try {
    return listProcess();
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
      return getInstanceProcess(id);
    } catch {
      console.error("Can't get running instance");
    }
  });
