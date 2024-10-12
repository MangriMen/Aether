import { createAsync } from '@solidjs/router';
import { Component, For, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { InstanceCard } from '@/entities/instance';
import {
  getMinecraftInstances,
  Instance,
  launchMinecraftInstance,
} from '@/entities/minecraft';

import { InstancesPanelProps } from './types';

export const InstancesPanel: Component<InstancesPanelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const instances = createAsync(() => getMinecraftInstances());

  const handleInstanceLaunch = (instance: Instance) => {
    launchMinecraftInstance(instance.path);
  };

  return (
    <div class={cn('flex gap-4', local.class)} {...others}>
      <For each={instances()}>
        {(instance) => (
          <InstanceCard
            instance={instance}
            onLaunchClick={() => handleInstanceLaunch(instance)}
          />
        )}
      </For>
    </div>
  );
};
