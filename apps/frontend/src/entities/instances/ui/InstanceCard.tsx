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
        `
          max-w-33 min-w-33 gap-2 rounded-md bg-card/card p-2 drop-shadow-md
          hover:bg-card/hover
          focus-visible:ring-ring
          active:animate-bump-out active:bg-card/active
          relative flex h-max cursor-pointer flex-col overflow-hidden border
          text-left outline-none
          focus-visible:ring-2 focus-visible:ring-offset-0
        `,
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
              `
                top-2 right-2 size-2.5 animate-pulse bg-warning fade-in-0
                absolute rounded-full
              `,
              local.class,
            )}
          />
        </Match>
        <Match when={local.isRunning}>
          <div
            class={cn(
              'top-2 right-2 size-2.5 bg-success absolute rounded-full',
              local.class,
            )}
          />
        </Match>
      </Switch>
    </Polymorphic>
  );
};
