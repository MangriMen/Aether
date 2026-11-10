import type { Component, ComponentProps } from 'solid-js';

import { For, Match, Show, splitProps, Switch } from 'solid-js';

import { useBrokenInstances, useInstances } from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { LoadError, SkeletonList, useArrowNavigation } from '@/shared/ui-kit';
import { cn, createDelayedFlag } from '@/shared/utils';

import { BrokenInstanceCard } from './BrokenInstanceCard';
import { InstanceControlledCard } from './InstanceControlledCard';

export type InstancesPanelProps = ComponentProps<'div'>;

export const InstancesPanel: Component<InstancesPanelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const instances = useInstances();
  const brokenInstances = useBrokenInstances();

  const { setRef, onKeyDown } = useArrowNavigation();

  // A cached or quick load would only flash the skeletons.
  const showSkeleton = createDelayedFlag(() => instances.isLoading);

  // The broken ones are shown whichever way the list of the readable ones went, so an
  // instance that cannot be read is not missing just because another one failed to load.
  const hasBroken = () => (brokenInstances.data?.length ?? 0) > 0;

  return (
    <div
      role='presentation'
      ref={setRef}
      onKeyDown={onKeyDown}
      class={cn('gap-4 flex flex-wrap', local.class)}
      {...others}
    >
      <Switch>
        <Match when={instances.isLoading}>
          <Show when={showSkeleton()}>
            <SkeletonList itemsCount={4} width={132} height={132} radius={6} />
          </Show>
        </Match>
        <Match when={instances.isError && !instances.data}>
          <LoadError
            class='m-auto'
            message={t('common.loadFailed')}
            retryLabel={t('common.retry')}
            onRetry={() => void instances.refetch()}
          />
        </Match>
        <Match when={instances.data?.length === 0 && !hasBroken()}>
          <p class='text-muted-foreground m-auto text-center whitespace-pre-line'>
            {t('home.noInstances')}
          </p>
        </Match>
        <Match when={instances.data}>
          <For each={instances.data}>
            {(instance) => <InstanceControlledCard instance={instance} />}
          </For>
        </Match>
      </Switch>
      <For each={brokenInstances.data}>
        {(instance) => <BrokenInstanceCard instance={instance} />}
      </For>
    </div>
  );
};
