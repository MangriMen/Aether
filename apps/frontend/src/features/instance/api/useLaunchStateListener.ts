import { onMount } from 'solid-js';

import {
  instanceCommands,
  instanceEvents,
  useEventListener,
} from '@/platform/ipc';
import { logError } from '@/shared/log';

/**
 * The launches that have the instances: the ones in progress on mount (the page may have been
 * reloaded while a launch waits for its pre-launch hook), then live events of their start and end.
 */
export const useLaunchStateListener = (
  callback: (instanceId: string, isLaunching: boolean) => void,
) => {
  // Instances told of by live events while the initial list is loading; the list must not
  // overwrite them. Not needed once it is applied.
  let liveInstanceIds: Set<string> | undefined = new Set<string>();

  useEventListener(instanceEvents.instanceEventDto, (e) => {
    const { event, instanceId } = e.payload;

    if (event !== 'launch_started' && event !== 'launch_ended') {
      return;
    }

    liveInstanceIds?.add(instanceId);
    callback(instanceId, event === 'launch_started');
  });

  onMount(async () => {
    try {
      for (const instanceId of await instanceCommands.listLaunching()) {
        if (!liveInstanceIds?.has(instanceId)) {
          callback(instanceId, true);
        }
      }
    } catch (error) {
      // The end of the launch is told by an event all the same, but one begun before the page
      // was reloaded shows as no launch until then.
      logError('Failed to list launching instances:', error);
    } finally {
      liveInstanceIds = undefined;
    }
  });
};
