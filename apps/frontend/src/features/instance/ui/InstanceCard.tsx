import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import { Polymorphic } from '@kobalte/core';
import { Match, mergeProps, splitProps, Switch } from 'solid-js';

import { cn } from '@/shared/utils';

import { type Instance } from '../model/instance';
import { INSTANCE_CARD_CLASS } from '../model/instanceCardClass';
import { InstanceIcon } from './InstanceIcon';
import { InstanceTitle } from './InstanceTitle';

export type InstanceCardProps = {
  instance: Instance;
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
    'isLoading',
    'isRunning',
    'overrideIcon',
    'class',
  ]);

  return (
    <Polymorphic class={cn(local.class, INSTANCE_CARD_CLASS)} {...others}>
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
