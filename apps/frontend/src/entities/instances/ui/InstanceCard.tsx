import type { PolymorphicProps } from '@kobalte/core';
import type { Component, ComponentProps, ValidComponent } from 'solid-js';

import { Polymorphic } from '@kobalte/core';
import { Match, mergeProps, splitProps, Switch } from 'solid-js';

import { cn } from '@/shared/lib';

import { InstanceIcon, type Instance } from '..';
import { InstanceTitle } from './InstanceTitle';

export type InstanceCardProps = {
  instance: Instance;
  instanceActionButton: Component<
    ComponentProps<'button'> & { instance: Instance }
  >;
  isLoading?: boolean;
  isRunning?: boolean;
  overrideIcon?: string;
  class?: string;
};

export const InstanceCard = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, InstanceCardProps>,
) => {
  const merged = mergeProps({ as: 'button' }, props);

  const [local, others] = splitProps(merged, [
    'instance',
    'instanceActionButton',
    'isLoading',
    'isRunning',
    'overrideIcon',
    'class',
  ]);

  return (
    <Polymorphic
      class={cn(
        local.class,
        'relative flex h-max max-w-[132px] min-w-[132px] cursor-pointer flex-col gap-2 overflow-hidden rounded-md border bg-card/card p-2 text-left drop-shadow-md outline-none hover:bg-card/hover focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 active:animate-bump-out active:bg-card/active',
      )}
      {...others}
    >
      <div class='flex items-center justify-center'>
        <InstanceIcon
          class='size-28'
          src={local.overrideIcon ?? local.instance.iconPath ?? undefined}
        />
      </div>
      <InstanceTitle
        class='select-none'
        name={local.instance.name}
        loader={local.instance.loader}
        gameVersion={local.instance.gameVersion}
      />
      <Switch>
        <Match when={local.isLoading}>
          <div
            class={cn(
              'absolute top-2 right-2 size-2.5 animate-pulse rounded-full bg-warning fade-in-0',
              local.class,
            )}
          />
        </Match>
        <Match when={local.isRunning}>
          <div
            class={cn(
              'absolute top-2 right-2 size-2.5 rounded-full bg-success',
              local.class,
            )}
          />
        </Match>
      </Switch>
    </Polymorphic>
  );
};
