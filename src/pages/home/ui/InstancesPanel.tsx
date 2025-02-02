import type { Component, ComponentProps } from 'solid-js';
import { createEffect, For, Match, Show, splitProps, Switch } from 'solid-js';

import { cn } from '@/shared/lib';

import { useInstances, refetchInstances } from '@/entities/instances';

import { InstanceActionButton } from '@/features/instance-action-button';

import { useTranslate } from '@/shared/model';

import { InstanceControlledCard } from './InstanceControlledCard';

export type InstancesPanelProps = ComponentProps<'div'>;

export const InstancesPanel: Component<InstancesPanelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslate();

  const instances = useInstances();

  createEffect(() => {
    refetchInstances();
  });

  return (
    <div class={cn('flex flex-wrap gap-4', local.class)} {...others}>
      <Switch>
        <Match when={instances?.() && !instances.loading}>
          <Show
            when={instances?.()?.[0]?.length}
            fallback={
              <p class='m-auto whitespace-pre-line text-center text-muted-foreground'>
                {t('home.noInstances')}
              </p>
            }
          >
            <For each={instances?.()?.[0]}>
              {(instance) => (
                <InstanceControlledCard
                  instance={instance}
                  instanceActionButton={InstanceActionButton}
                />
              )}
            </For>
          </Show>
        </Match>
      </Switch>
    </div>
  );
};
