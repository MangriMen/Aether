import {
  Component,
  createEffect,
  For,
  Match,
  Show,
  splitProps,
  Switch,
} from 'solid-js';

import { cn } from '@/shared/lib';

import { getInstances, refetchInstances } from '@/entities/instance';

import { InstanceControlledCard } from '@/features/instance-controlled-card';

import { InstancesPanelProps } from './types';

export const InstancesPanel: Component<InstancesPanelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const instances = getInstances();

  createEffect(() => {
    refetchInstances();
  });

  return (
    <div class={cn('flex flex-wrap gap-4', local.class)} {...others}>
      <Switch>
        {/* <Match when={instances.loading}>
          <ProgressCircle />
        </Match> */}
        <Match when={!instances.loading}>
          <Show
            when={instances()?.length}
            fallback={<span class='m-auto'>You don't have instances</span>}
          >
            <For each={instances()}>
              {(instance) => <InstanceControlledCard instance={instance} />}
            </For>
          </Show>
        </Match>
      </Switch>
    </div>
  );
};
