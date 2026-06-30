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
        'text-left flex min-w-[132px] outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 max-w-[132px] flex-col cursor-pointer gap-2 bg-card/card hover:bg-card/hover active:bg-card/active drop-shadow-md border rounded-md p-2 h-max overflow-hidden relative active:animate-bump-out',
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
              'size-2.5 rounded-full bg-warning absolute right-2 top-2 animate-pulse fade-in-0',
              local.class,
            )}
          />
        </Match>
        <Match when={local.isRunning}>
          <div
            class={cn(
              'size-2.5 rounded-full bg-success absolute right-2 top-2',
              local.class,
            )}
          />
        </Match>
      </Switch>
    </Polymorphic>
  );
};
