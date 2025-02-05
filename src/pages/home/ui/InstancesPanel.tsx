import type { Component, ComponentProps } from 'solid-js';
import {
  createEffect,
  createMemo,
  For,
  Match,
  Show,
  splitProps,
  Switch,
} from 'solid-js';

import { cn } from '@/shared/lib';

import { refetchInstances, useInstances } from '@/entities/instances';

import { InstanceActionButton } from '@/features/instance-action-button';

import { useTranslate } from '@/shared/model';

import { InstanceControlledCard } from './InstanceControlledCard';

export type InstancesPanelProps = ComponentProps<'div'>;

export const InstancesPanel: Component<InstancesPanelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslate();

  const [instances, { isLoading }] = useInstances();

  const instancesArray = createMemo(() => Array.from(instances.values()));

  createEffect(() => {
    refetchInstances();
  });

  return (
    <div class={cn('flex flex-wrap gap-4', local.class)} {...others}>
      <Switch>
        <Match when={instances.size && !isLoading()}>
          <Show
            when={instances.size}
            fallback={
              <p class='m-auto whitespace-pre-line text-center text-muted-foreground'>
                {t('home.noInstances')}
              </p>
            }
          >
            <For each={instancesArray()}>
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
