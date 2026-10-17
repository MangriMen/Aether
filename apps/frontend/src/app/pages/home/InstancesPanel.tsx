import type { Component, ComponentProps } from 'solid-js';

import { For, Match, Show, splitProps, Switch } from 'solid-js';

import { InstanceActionButton, useInstances } from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { useArrowNavigation } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

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
      class={cn('gap-4 flex flex-wrap', local.class)}
      {...others}
    >
      <Switch>
        <Match when={instances.data?.length && !instances.isLoading}>
          <Show
            when={instances.data?.length}
            fallback={
              <p
                class='
                  text-muted-foreground m-auto text-center whitespace-pre-line
                '
              >
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
