import type { Component, ComponentProps } from 'solid-js';

import { Match, splitProps, Switch } from 'solid-js';

import { cn } from '@/shared/lib';
import { Image } from '@/shared/ui';

import { type Instance } from '..';
import { InstanceTitle } from './InstanceTitle';

export type InstanceCardProps = ComponentProps<'div'> & {
  instance: Instance;
  instanceActionButton: Component<
    ComponentProps<'button'> & { instance: Instance }
  >;
  isLoading?: boolean;
  isRunning?: boolean;
  overrideIcon?: string;
};

export const InstanceCard: Component<InstanceCardProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'instanceActionButton',
    'isLoading',
    'isRunning',
    'overrideIcon',
    'class',
  ]);

  return (
    <div
      class={cn(
        local.class,
        'group w-[132px] flex flex-col cursor-pointer gap-2 bg-card/card hover:bg-card/hover active:bg-card/active drop-shadow-md border rounded-md p-2 h-max overflow-hidden relative active:animate-bump-out',
      )}
      {...others}
    >
      <Image
        class='mx-auto size-24 min-w-max border-none bg-transparent'
        src={local.overrideIcon ?? local.instance.iconPath ?? undefined}
      />
      <InstanceTitle
        name={local.instance.name}
        loader={local.instance.loader}
        gameVersion={local.instance.gameVersion}
      />
      <Switch>
        <Match when={local.isLoading}>
          <div
            class={cn(
              'size-2.5 rounded-full bg-yellow-400 absolute right-2 top-2 animate-pulse fade-in-0',
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
      <local.instanceActionButton
        class='absolute right-4 top-[72px] p-0 opacity-0 transition-[bottom,opacity] focus-within:bottom-1/4 focus-within:opacity-100 disabled:opacity-0 group-hover:bottom-1/4 group-hover:opacity-100'
        instance={local.instance}
      />
    </div>
  );
};
