import {
  Component,
  createEffect,
  createResource,
  For,
  Match,
  Show,
  splitProps,
  Switch,
} from 'solid-js';

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

  const [instances, { refetch }] = createResource(
    () => getMinecraftInstances(),
    {
      initialValue: [],
    },
  );

  const handleInstanceLaunch = (instance: Instance) => {
    launchMinecraftInstance(instance.path);
  };

  createEffect(() => {
    refetch();
  });

  return (
    <div class={cn('flex flex-wrap gap-4', local.class)} {...others}>
      <Switch>
        <Match when={instances.loading}>Loading</Match>
        <Match when={!instances.loading}>
          <Show
            when={instances()?.length}
            fallback={<span class='m-auto'>You don't have instances</span>}
          >
            <For each={instances()}>
              {(instance) => (
                <InstanceCard
                  instance={instance}
                  onLaunchClick={() => handleInstanceLaunch(instance)}
                />
              )}
            </For>
          </Show>
        </Match>
      </Switch>
    </div>
  );
};
