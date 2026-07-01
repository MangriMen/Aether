import type { Component, ComponentProps } from 'solid-js';

import { For, Match, Show, splitProps, Switch } from 'solid-js';

import { useInstances } from '@/entities/instances';
import { InstanceActionButton } from '@/features/instance-action-button';
import { cn, useArrowNavigation } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { InstanceControlledCard } from './InstanceControlledCard';

export type InstancesPanelProps = ComponentProps<'div'>;

export const InstancesPanel: Component<InstancesPanelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const instances = useInstances();

  const { setRef, onKeyDown } = useArrowNavigation();

  return (
    <div
      role='presentation'
      ref={setRef}
      onKeyDown={onKeyDown}
      class={cn('flex flex-wrap gap-4', local.class)}
      {...others}
    >
      <Switch>
        <Match when={instances.data?.length && !instances.isLoading}>
          <Show
            when={instances.data?.length}
            fallback={
              <p class='m-auto text-center whitespace-pre-line text-muted-foreground'>
                {t('home.noInstances')}
              </p>
            }
          >
            <For each={instances.data}>
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
