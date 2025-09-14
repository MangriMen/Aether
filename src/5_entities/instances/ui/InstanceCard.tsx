import type { Component, ComponentProps } from 'solid-js';

import { Match, splitProps, Switch } from 'solid-js';

import { type Instance } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { Image } from '@/shared/ui';

import { InstanceTitle } from './InstanceTitle';

export type InstanceCardProps = {
  instance: Instance;
  instanceActionButton: Component<
    { instance: Instance } & ComponentProps<'button'>
  >;
  isLoading?: boolean;
  isRunning?: boolean;
} & ComponentProps<'div'>;

export const InstanceCard: Component<InstanceCardProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'instanceActionButton',
    'isLoading',
    'isRunning',
    'class',
  ]);

  return (
    <div
      class={cn(
        local.class,
        'group flex flex-col cursor-pointer gap-2 border rounded-md p-2 size-max overflow-hidden relative active:animate-bump-out',
      )}
      {...others}
    >
      <Image src={local.instance.iconPath} />
      <InstanceTitle
        gameVersion={local.instance.gameVersion}
        loader={local.instance.loader}
        name={local.instance.name}
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
        class='absolute bottom-1/3 left-1/2 p-0 pr-0.5 opacity-0 transition-[bottom,opacity] focus-within:bottom-1/4 focus-within:opacity-100 disabled:opacity-0 group-hover:bottom-1/4 group-hover:opacity-100'
        instance={local.instance}
      />
    </div>
  );
};
