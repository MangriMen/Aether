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

import { useInstances, refetchInstances } from '@/entities/instance';

import { InstanceControlledCard } from '@/features/instance-controlled-card';

import { InstancesPanelProps } from './types';

export const InstancesPanel: Component<InstancesPanelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const instances = useInstances();

  createEffect(() => {
    refetchInstances();
  });

  return (
    <div class={cn('flex flex-wrap gap-4 h-full', local.class)} {...others}>
      <Switch>
        <Match when={instances?.() && !instances.loading}>
          <Show
            when={instances?.()?.[0]?.length}
            fallback={
              <p class='m-auto text-center text-muted-foreground'>
                <span>No instances available</span>
                <br />
                <span>It seems you don`t have any instances at the moment</span>
              </p>
            }
          >
            <For each={instances?.()?.[0]}>
              {(instance) => <InstanceControlledCard instance={instance} />}
            </For>
          </Show>
        </Match>
      </Switch>
    </div>
  );
};
